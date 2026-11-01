use crate::clipboard;
use crate::db::{clean_path_str, ensure_db, get_active_emotes_dir, normalize_path};
use crate::models::Emote;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, GenericImageView, ImageFormat};
use rusqlite::{params, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
pub fn list_emotes() -> Result<Vec<Emote>, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare("SELECT id, name, tag, path, original_width, original_height, format, created_at FROM emotes ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let emote_iter = stmt
        .query_map([], |row| {
            let raw_path: String = row.get(3)?;
            let clean_path = clean_path_str(raw_path);
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
            })
        })
        .map_err(|e| e.to_string())?;

    let mut emotes = Vec::new();
    for em in emote_iter {
        emotes.push(em.map_err(|e| e.to_string())?);
    }
    Ok(emotes)
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

    let mut paths = Vec::new();
    let entries = fs::read_dir(p).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
                    paths.push(entry_path.to_string_lossy().to_string());
                }
            }
        }
    }

    import_emotes_internal(paths, true)
}

fn import_emotes_internal(paths: Vec<String>, convert_to_webp: bool) -> Result<Vec<Emote>, String> {
    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let abs_dir = dir.canonicalize().unwrap_or(dir);

    let mut created = Vec::new();
    for p in paths {
        let src = Path::new(&p);
        if !src.exists() || !src.is_file() {
            continue;
        }

        let orig_ext = src
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();

        // Detectar si es un GIF animado (>1 fotograma) para conservar su secuencia intacta
        let mut is_animated_gif = false;
        let mut gif_dimensions: Option<(u32, u32)> = None;
        if orig_ext == "gif" {
            if let Ok(file) = fs::File::open(src) {
                let reader = std::io::BufReader::new(file);
                if let Ok(decoder) = GifDecoder::new(reader) {
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

        let target_ext = if is_animated_gif {
            "gif".to_string()
        } else if convert_to_webp {
            "webp".to_string()
        } else {
            orig_ext.clone()
        };

        let file_name = format!("{}.{}", uuid::Uuid::new_v4(), target_ext);
        let dest = abs_dir.join(&file_name);

        let (w, h, format) = if is_animated_gif {
            fs::copy(src, &dest).map_err(|e| e.to_string())?;
            let (w, h) = gif_dimensions.unwrap_or_else(|| {
                image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0))
            });
            (w as i32, h as i32, "gif".to_string())
        } else if convert_to_webp && orig_ext != "webp" {
            let img = image::open(src).map_err(|e| format!("Error abriendo {:?}: {}", src, e))?;
            let (w, h) = img.dimensions();
            let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
            img.write_to(&mut file, ImageFormat::WebP)
                .map_err(|e| format!("Error convirtiendo a WebP: {}", e))?;
            (w as i32, h as i32, "webp".to_string())
        } else {
            fs::copy(src, &dest).map_err(|e| e.to_string())?;
            match image::open(&dest) {
                Ok(img) => {
                    let (w, h) = img.dimensions();
                    let fmt = match ImageFormat::from_extension(&orig_ext) {
                        Some(f) => format!("{:?}", f).to_lowercase(),
                        None => orig_ext.clone(),
                    };
                    (w as i32, h as i32, fmt)
                }
                Err(_) => (0, 0, orig_ext.clone()),
            }
        };

        let now = chrono::Utc::now().to_rfc3339();
        let dest_str = normalize_path(&dest);

        let emote_name = src
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("emote")
            .to_string();

        conn.execute(
            "INSERT INTO emotes (name, tag, path, original_width, original_height, format, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![emote_name, Option::<String>::None, dest_str, w, h, format, now],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();
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
        });
    }
    Ok(created)
}

#[tauri::command]
pub fn delete_emote(id: i64) -> Result<bool, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare("SELECT path FROM emotes WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let path: Option<String> = stmt
        .query_row([id], |r| r.get(0))
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
    path: String,
    format: String,
    target_width: u32,
    target_height: u32,
) -> Result<bool, String> {
    clipboard::copy_emote(&path, &format, target_width, target_height)
}

