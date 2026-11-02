use crate::clipboard::{self, ClipboardPayload};
use crate::db::{
    clean_path_str, ensure_db, get_active_db_dir, get_active_emotes_dir, normalize_path,
    set_active_db_dir,
};
use crate::models::{Emote, ResyncStats};
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, GenericImageView, ImageFormat};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn save_raw_bytes_as_emote(bytes: &[u8], emote_name: &str) -> Result<Emote, String> {
    if bytes.is_empty() {
        return Err("Los datos de la imagen están vacíos".to_string());
    }

    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let abs_dir = dir.canonicalize().unwrap_or(dir);

    let guessed = image::guess_format(bytes)
        .map_err(|e| format!("El contenido no es una imagen válida: {}", e))?;

    let mut is_animated_gif = false;
    let mut gif_dimensions: Option<(u32, u32)> = None;

    if guessed == ImageFormat::Gif {
        if let Ok(decoder) = GifDecoder::new(Cursor::new(bytes)) {
            if let Ok(frames) = decoder.into_frames().collect_frames() {
                if frames.len() > 1 {
                    is_animated_gif = true;
                    if let Some(first) = frames.first() {
                        gif_dimensions = Some((first.buffer().width(), first.buffer().height()));
                    }
                }
            }
        }
    }

    let target_ext = if is_animated_gif { "gif" } else { "webp" };
    let dest = abs_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), target_ext));

    let (w, h, format) = if is_animated_gif {
        fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        let (w, h) = gif_dimensions.unwrap_or_else(|| {
            image::load_from_memory(bytes)
                .map(|i| i.dimensions())
                .unwrap_or((0, 0))
        });
        (w as i32, h as i32, "gif".to_string())
    } else if guessed == ImageFormat::WebP {
        let (w, h) = image::load_from_memory(bytes)
            .map(|i| i.dimensions())
            .map_err(|e| format!("Error leyendo imagen WebP: {}", e))?;
        fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        (w as i32, h as i32, "webp".to_string())
    } else {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("Error decodificando imagen: {}", e))?;
        let (w, h) = img.dimensions();
        let mut out_file = fs::File::create(&dest).map_err(|e| e.to_string())?;
        img.write_to(&mut out_file, ImageFormat::WebP)
            .map_err(|e| format!("Error convirtiendo a WebP: {}", e))?;
        (w as i32, h as i32, "webp".to_string())
    };

    let now = chrono::Utc::now().to_rfc3339();
    let dest_str = normalize_path(&dest);
    let clean_name = if emote_name.trim().is_empty() {
        "emote".to_string()
    } else {
        emote_name.trim().to_string()
    };

    let id = insert_emote_record(&conn, &clean_name, &dest_str, w, h, &format, &now)?;
    let size = fs::metadata(&dest).map(|m| m.len()).ok();

    Ok(Emote {
        id,
        name: clean_name,
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
    })
}

fn infer_name_from_url(url: &str) -> String {
    let without_query = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);
    let last_segment = without_query
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("emote");
    let stem = Path::new(last_segment)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("emote")
        .trim();
    if stem.is_empty() {
        "emote".to_string()
    } else {
        stem.to_string()
    }
}

#[tauri::command]
pub fn list_emotes() -> Result<Vec<Emote>, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.name, e.tag, e.path, e.original_width, e.original_height, e.format, e.created_at,
                    COALESCE(v.copy_count, 0), COALESCE(v.interaction_count, 0), v.last_used_at
             FROM emotes e
             LEFT JOIN emote_votes v ON v.emote_id = e.id
             ORDER BY e.created_at DESC, e.id DESC",
        )
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
pub fn set_storage_path(path: String, migrate: Option<bool>) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(|e| format!("No se pudo crear la carpeta: {}", e))?;
    }
    let abs = p.canonicalize().unwrap_or(p);
    let abs_str = normalize_path(&abs);

    if abs_str.contains("src-tauri") {
        return Err(
            "No se puede ubicar la carpeta de emotes dentro de src-tauri".to_string(),
        );
    }

    let should_migrate = migrate.unwrap_or(false);
    let mut should_migrate_db = false;

    {
        let conn = ensure_db()?;
        let old_dir = get_active_emotes_dir(&conn);
        let old_abs = old_dir.canonicalize().unwrap_or(old_dir);
        let old_str = normalize_path(&old_abs);

        let cur_db_dir = get_active_db_dir();
        let cur_db_abs = cur_db_dir.canonicalize().unwrap_or(cur_db_dir);
        let cur_db_str = normalize_path(&cur_db_abs);

        if should_migrate && old_str.to_lowercase() != abs_str.to_lowercase() {
            if cur_db_str.to_lowercase() == old_str.to_lowercase() {
                should_migrate_db = true;
            }

            // 1. Migrar archivos registrados en la tabla emotes y actualizar sus rutas
            let rows: Vec<(i64, String)> = {
                let mut stmt = conn
                    .prepare("SELECT id, path FROM emotes")
                    .map_err(|e| e.to_string())?;
                let mapped = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(|e| e.to_string())?;
                mapped.flatten().collect()
            };

            for (id, raw_path) in rows {
                let clean = clean_path_str(&raw_path);
                let src_path = Path::new(&clean);
                if let Some(file_name) = src_path.file_name() {
                    let dest_path = abs.join(file_name);
                    let dest_str = normalize_path(&dest_path);

                    if src_path.exists() && src_path != dest_path {
                        if fs::copy(src_path, &dest_path).is_ok() {
                            let _ = fs::remove_file(src_path);
                        }
                    }

                    let _ = conn.execute(
                        "UPDATE emotes SET path = ?1 WHERE id = ?2",
                        params![dest_str, id],
                    );
                }
            }

            // 2. Migrar imágenes sueltas que estuvieran en la carpeta anterior
            if old_abs.exists() && old_abs.is_dir() {
                if let Ok(entries) = fs::read_dir(&old_abs) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if !entry_path.is_file() {
                            continue;
                        }
                        let ext = entry_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
                            if let Some(fname) = entry_path.file_name() {
                                let dest_path = abs.join(fname);
                                if entry_path != dest_path && !dest_path.exists() {
                                    if fs::copy(&entry_path, &dest_path).is_ok() {
                                        let _ = fs::remove_file(&entry_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('storage_path', ?1)",
            params![&abs_str],
        )
        .map_err(|e| e.to_string())?;
    }

    if should_migrate_db {
        let _ = set_active_db_dir(&abs_str, true)?;
    }

    Ok(abs_str)
}

#[tauri::command]
pub fn get_db_path() -> Result<String, String> {
    let dir = get_active_db_dir();
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    let abs = dir.canonicalize().unwrap_or(dir);
    Ok(normalize_path(&abs))
}

#[tauri::command]
pub fn set_db_path(path: String, migrate: Option<bool>) -> Result<String, String> {
    set_active_db_dir(&path, migrate.unwrap_or(true))
}

fn import_single_file(
    conn: &Connection,
    abs_dir: &Path,
    src: &Path,
) -> Result<Option<Emote>, String> {
    if !src.is_file() {
        return Ok(None);
    }

    let orig_ext = match src.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext.to_lowercase(),
        None => return Ok(None),
    };

    if !matches!(orig_ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
        return Ok(None);
    }

    // Detectar si es un GIF animado (>1 fotograma) para conservar su secuencia intacta
    let mut is_animated_gif = false;
    let mut gif_dimensions: Option<(u32, u32)> = None;
    if orig_ext == "gif" {
        if let Ok(file) = fs::File::open(src) {
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
        fs::copy(src, &dest).map_err(|e| e.to_string())?;
        let (w, h) = gif_dimensions.unwrap_or_else(|| {
            image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0))
        });
        (w as i32, h as i32, "gif".to_string())
    } else if orig_ext != "webp" {
        let img = image::open(src).map_err(|e| format!("Error abriendo {:?}: {}", src, e))?;
        let (w, h) = img.dimensions();
        let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
        img.write_to(&mut file, ImageFormat::WebP)
            .map_err(|e| format!("Error convirtiendo a WebP: {}", e))?;
        (w as i32, h as i32, "webp".to_string())
    } else {
        fs::copy(src, &dest).map_err(|e| e.to_string())?;
        let (w, h) = image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0));
        (w as i32, h as i32, "webp".to_string())
    };

    let now = chrono::Utc::now().to_rfc3339();
    let dest_str = normalize_path(&dest);
    let emote_name = extract_stem(src);

    let id = insert_emote_record(conn, &emote_name, &dest_str, w, h, &format, &now)?;
    let size = fs::metadata(&dest).map(|m| m.len()).ok();
    Ok(Some(Emote {
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
    }))
}

#[tauri::command]
pub fn import_paths(paths: Vec<String>) -> Result<Vec<Emote>, String> {
    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let abs_dir = dir.canonicalize().unwrap_or(dir);

    let mut created = Vec::new();

    for raw in paths {
        let p = Path::new(&raw);
        if !p.exists() {
            continue;
        }
        if p.is_dir() {
            let entries = fs::read_dir(p).map_err(|e| e.to_string())?;
            for entry in entries.flatten() {
                if let Some(emote) = import_single_file(&conn, &abs_dir, &entry.path())? {
                    created.push(emote);
                }
            }
        } else if p.is_file() {
            if let Some(emote) = import_single_file(&conn, &abs_dir, p)? {
                created.push(emote);
            }
        }
    }

    Ok(created)
}

#[tauri::command]
pub fn import_raw_bytes(bytes: Vec<u8>, name: Option<String>) -> Result<Emote, String> {
    let clean_name = name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("emote");
    save_raw_bytes_as_emote(&bytes, clean_name)
}

#[tauri::command]
pub fn import_from_url(url: String, name: Option<String>) -> Result<Emote, String> {
    let clean_url = url.trim();
    if !clean_url.starts_with("http://") && !clean_url.starts_with("https://") {
        return Err("La URL debe comenzar con http:// o https://".to_string());
    }

    let mut cmd = Command::new("curl");
    cmd.args([
        "-L",
        "-s",
        "-S",
        "-f",
        "--max-time",
        "25",
        "-A",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) StickUp/1.0",
        clean_url,
    ]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("No se pudo descargar desde la URL: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error al descargar la URL: {}", stderr.trim()));
    }

    let emote_name = name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| infer_name_from_url(clean_url));

    save_raw_bytes_as_emote(&output.stdout, &emote_name)
}

#[tauri::command]
pub fn import_from_clipboard(name: Option<String>) -> Result<Emote, String> {
    let custom_name = name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    match clipboard::read_clipboard_payload()? {
        ClipboardPayload::FilePath(path) => {
            let bytes = fs::read(&path)
                .map_err(|e| format!("No se pudo leer el archivo del portapapeles: {}", e))?;
            let final_name = custom_name.unwrap_or_else(|| extract_stem(&path));
            save_raw_bytes_as_emote(&bytes, &final_name)
        }
        ClipboardPayload::RawImageBytes(bytes) => {
            let final_name = custom_name.unwrap_or_else(|| {
                format!("clipboard_{}", &uuid::Uuid::new_v4().to_string()[..6])
            });
            save_raw_bytes_as_emote(&bytes, &final_name)
        }
        ClipboardPayload::RgbaImage {
            rgba,
            width,
            height,
        } => {
            let img_buf = image::RgbaImage::from_raw(width, height, rgba)
                .ok_or_else(|| "Buffer RGBA inválido en el portapapeles".to_string())?;
            let dyn_img = image::DynamicImage::ImageRgba8(img_buf);
            let mut buf = Cursor::new(Vec::new());
            dyn_img
                .write_to(&mut buf, ImageFormat::WebP)
                .map_err(|e| format!("Error codificando imagen del portapapeles: {}", e))?;
            let final_name = custom_name.unwrap_or_else(|| {
                format!("clipboard_{}", &uuid::Uuid::new_v4().to_string()[..6])
            });
            save_raw_bytes_as_emote(&buf.into_inner(), &final_name)
        }
        ClipboardPayload::TextUrl(url) => import_from_url(url, custom_name),
    }
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
    let _ = conn.execute("DELETE FROM emote_votes WHERE emote_id = ?1", params![id]);
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
                    "INSERT INTO emote_votes (emote_id, copy_count, interaction_count, score, last_used_at)
                     VALUES (?1, 1, 0, 1, ?2)
                     ON CONFLICT(emote_id) DO UPDATE SET
                         copy_count = copy_count + 1,
                         score = copy_count + 1,
                         last_used_at = excluded.last_used_at",
                    params![emote_id, now],
                );
            }
        }
    }
    Ok(copied)
}

#[tauri::command]
pub fn record_interaction(id: i64) -> Result<bool, String> {
    let conn = ensure_db()?;
    conn.execute(
        "INSERT INTO emote_votes (emote_id, copy_count, interaction_count, score, last_used_at)
         VALUES (?1, 0, 1, 0, NULL)
         ON CONFLICT(emote_id) DO UPDATE SET
             interaction_count = interaction_count + 1",
        params![id],
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
                let _ = conn.execute("DELETE FROM emote_votes WHERE emote_id = ?1", params![id]);
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
