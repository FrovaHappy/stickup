use crate::clipboard;
use crate::db::{clean_path_str, ensure_db, get_active_emotes_dir, normalize_path};
use crate::models::{Emote, ResyncStats};
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, GenericImageView, ImageFormat};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn extract_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("emote")
        .to_string()
}

fn insert_emote_record(
    conn: &Connection,
    name: &str,
    path: &str,
    w: i32,
    h: i32,
    format: &str,
    now: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO emotes (name, tag, path, original_width, original_height, format, created_at) VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6)",
        params![name, path, w, h, format, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn list_emotes() -> Result<Vec<Emote>, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare("SELECT id, name, tag, path, original_width, original_height, format, created_at, COALESCE(copy_count, 0), COALESCE(interaction_count, 0), last_used_at FROM emotes ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let emote_iter = stmt
        .query_map([], |row| {
            let raw_path: String = row.get(3)?;
            let clean_path = clean_path_str(&raw_path);
            let size = fs::metadata(&clean_path).map(|m| m.len()).ok();
            Ok(Emote {
                id: row.get(0)?,
                name: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                tag: row.get(2)?,
                path: clean_path,
                original_width: row.get(4)?,
                original_height: row.get(5)?,
                format: row.get(6)?,
                created_at: row.get(7)?,
                file_size: size,
                copy_count: row.get(8)?,
                interaction_count: row.get(9)?,
                last_used_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;

    emote_iter
        .collect::<Result<Vec<Emote>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_storage_path() -> Result<String, String> {
    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    let abs = dir.canonicalize().unwrap_or(dir);
    Ok(normalize_path(&abs))
}

#[tauri::command]
pub fn set_storage_path(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(|e| format!("No se pudo crear la carpeta: {}", e))?;
    }
    let abs = p.canonicalize().unwrap_or(p);
    let abs_str = normalize_path(&abs);

    let conn = ensure_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('storage_path', ?1)",
        params![&abs_str],
    )
    .map_err(|e| e.to_string())?;

    Ok(abs_str)
}

#[tauri::command]
pub fn import_folder(folder_path: String) -> Result<Vec<Emote>, String> {
    let p = Path::new(&folder_path);
    if !p.exists() || !p.is_dir() {
        return Err(format!("La carpeta no existe o no es válida: {}", folder_path));
    }

    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let abs_dir = dir.canonicalize().unwrap_or(dir);

    let mut created = Vec::new();
    let entries = fs::read_dir(p).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let src = entry.path();
        if !src.is_file() {
            continue;
        }

        let orig_ext = match src.extension().and_then(|e| e.to_str()) {
            Some(ext) => ext.to_lowercase(),
            None => continue,
        };

        if !matches!(orig_ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
            continue;
        }

        // Detectar si es un GIF animado (>1 fotograma) para conservar su secuencia intacta
        let mut is_animated_gif = false;
        let mut gif_dimensions: Option<(u32, u32)> = None;
        if orig_ext == "gif" {
            if let Ok(file) = fs::File::open(&src) {
                if let Ok(decoder) = GifDecoder::new(std::io::BufReader::new(file)) {
                    if let Ok(frames) = decoder.into_frames().collect_frames() {
                        if frames.len() > 1 {
                            is_animated_gif = true;
                            if let Some(first) = frames.first() {
                                gif_dimensions =
                                    Some((first.buffer().width(), first.buffer().height()));
                            }
                        }
                    }
                }
            }
        }

        let target_ext = if is_animated_gif { "gif" } else { "webp" };
        let dest = abs_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), target_ext));

        let (w, h, format) = if is_animated_gif {
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            let (w, h) = gif_dimensions.unwrap_or_else(|| {
                image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0))
            });
            (w as i32, h as i32, "gif".to_string())
        } else if orig_ext != "webp" {
            let img = image::open(&src).map_err(|e| format!("Error abriendo {:?}: {}", src, e))?;
            let (w, h) = img.dimensions();
            let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
            img.write_to(&mut file, ImageFormat::WebP)
                .map_err(|e| format!("Error convirtiendo a WebP: {}", e))?;
            (w as i32, h as i32, "webp".to_string())
        } else {
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            let (w, h) = image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0));
            (w as i32, h as i32, "webp".to_string())
        };

        let now = chrono::Utc::now().to_rfc3339();
        let dest_str = normalize_path(&dest);
        let emote_name = extract_stem(&src);

        let id = insert_emote_record(&conn, &emote_name, &dest_str, w, h, &format, &now)?;
        let size = fs::metadata(&dest).map(|m| m.len()).ok();
        created.push(Emote {
            id,
            name: emote_name,
            tag: None,
            path: dest_str,
            original_width: w,
            original_height: h,
            format,
            created_at: now,
            file_size: size,
            copy_count: 0,
            interaction_count: 0,
            last_used_at: None,
        });
    }
    Ok(created)
}

#[tauri::command]
pub fn delete_emote(id: i64) -> Result<bool, String> {
    let conn = ensure_db()?;
    let path: Option<String> = conn
        .query_row("SELECT path FROM emotes WHERE id = ?1", [id], |r| r.get(0))
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(p) = path {
        let _ = fs::remove_file(Path::new(&p));
    }
    conn.execute("DELETE FROM emotes WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn update_tag(id: i64, name: Option<String>, tag: Option<String>) -> Result<bool, String> {
    let conn = ensure_db()?;
    match (name, tag) {
        (Some(n), Some(t)) => {
            conn.execute(
                "UPDATE emotes SET name = ?1, tag = ?2 WHERE id = ?3",
                params![n, t, id],
            )
            .map_err(|e| e.to_string())?;
        }
        (Some(n), None) => {
            conn.execute("UPDATE emotes SET name = ?1 WHERE id = ?2", params![n, id])
                .map_err(|e| e.to_string())?;
        }
        (None, Some(t)) => {
            conn.execute("UPDATE emotes SET tag = ?1 WHERE id = ?2", params![t, id])
                .map_err(|e| e.to_string())?;
        }
        (None, None) => {}
    }
    Ok(true)
}

#[tauri::command]
pub fn copy_to_clipboard(
    id: Option<i64>,
    path: String,
    format: String,
    target_width: u32,
    target_height: u32,
) -> Result<bool, String> {
    let copied = clipboard::copy_emote(&path, &format, target_width, target_height)?;
    if copied {
        if let Some(emote_id) = id {
            if let Ok(conn) = ensure_db() {
                let now = chrono::Utc::now().to_rfc3339();
                let _ = conn.execute(
                    "UPDATE emotes SET copy_count = COALESCE(copy_count, 0) + 1, interaction_count = COALESCE(interaction_count, 0) + 1, last_used_at = ?1 WHERE id = ?2",
                    params![now, emote_id],
                );
            }
        }
    }
    Ok(copied)
}

#[tauri::command]
pub fn record_interaction(id: i64) -> Result<bool, String> {
    let conn = ensure_db()?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE emotes SET interaction_count = COALESCE(interaction_count, 0) + 1, last_used_at = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn resync_emotes() -> Result<ResyncStats, String> {
    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let abs_dir = dir.canonicalize().unwrap_or(dir);

    let mut stats = ResyncStats::default();

    // 1. Limpiar registros huérfanos en la BD y mapear las rutas existentes
    let mut existing_db: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, path FROM emotes")
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .flatten()
            .collect();

        for (id, raw_path) in rows {
            let clean = clean_path_str(&raw_path);
            if !Path::new(&clean).exists() {
                let _ = conn.execute("DELETE FROM emotes WHERE id = ?1", params![id]);
                stats.removed_missing_from_db += 1;
            } else {
                existing_db.insert(clean.to_lowercase(), id);
            }
        }
    }

    // 2. Escanear el directorio activo de emotes
    let entries = fs::read_dir(&abs_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let ext = match entry_path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue,
        };

        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
            continue;
        }

        stats.scanned_files += 1;
        let orig_path_str = normalize_path(&entry_path);
        let orig_key = orig_path_str.to_lowercase();

        if matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
            let orig_size = fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0);
            let img = match image::open(&entry_path) {
                Ok(i) => i,
                Err(_) => continue,
            };
            let (w, h) = img.dimensions();

            let mut dest_path = entry_path.with_extension("webp");
            if dest_path.exists() && dest_path != entry_path {
                dest_path = abs_dir.join(format!(
                    "{}_{}.webp",
                    extract_stem(&entry_path),
                    &uuid::Uuid::new_v4().to_string()[..8]
                ));
            }

            let mut out_file = fs::File::create(&dest_path).map_err(|e| e.to_string())?;
            if img.write_to(&mut out_file, ImageFormat::WebP).is_ok() {
                drop(out_file);
                let new_size = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
                let new_path_str = normalize_path(&dest_path);

                if dest_path != entry_path {
                    let _ = fs::remove_file(&entry_path);
                }

                stats.converted_to_webp += 1;
                if ext == "png" {
                    stats.png_converted += 1;
                } else {
                    stats.jpg_converted += 1;
                }
                stats.bytes_before += orig_size;
                stats.bytes_after += new_size;

                if let Some(&id) = existing_db.get(&orig_key) {
                    let _ = conn.execute(
                        "UPDATE emotes SET path = ?1, format = 'webp', original_width = ?2, original_height = ?3 WHERE id = ?4",
                        params![new_path_str, w as i32, h as i32, id],
                    );
                    existing_db.remove(&orig_key);
                    existing_db.insert(new_path_str.to_lowercase(), id);
                } else {
                    let now = chrono::Utc::now().to_rfc3339();
                    let _ = insert_emote_record(
                        &conn,
                        &extract_stem(&entry_path),
                        &new_path_str,
                        w as i32,
                        h as i32,
                        "webp",
                        &now,
                    );
                    stats.added_to_db += 1;
                }
            }
        } else if !existing_db.contains_key(&orig_key) {
            let (w, h) = image::open(&entry_path)
                .map(|i| i.dimensions())
                .unwrap_or((0, 0));
            let now = chrono::Utc::now().to_rfc3339();
            let _ = insert_emote_record(
                &conn,
                &extract_stem(&entry_path),
                &orig_path_str,
                w as i32,
                h as i32,
                &ext,
                &now,
            );
            stats.added_to_db += 1;
        }
    }

    stats.saved_bytes = (stats.bytes_before as i64) - (stats.bytes_after as i64);

    Ok(stats)
}
