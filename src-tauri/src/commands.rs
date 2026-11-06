use crate::clipboard::{self, ClipboardPayload};
use crate::db::{
    self, clean_path_str, ensure_db, get_active_db_dir, get_active_emotes_dir, normalize_path,
    set_active_db_dir, upgrade_legacy_db_in_dir, DB_FILE_NAME,
};
use crate::models::{BackupInfo, BackupResult, Emote, RestoreResult, ResyncStats};
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::codecs::webp::WebPDecoder;
use image::{AnimationDecoder, Delay, Frame, GenericImageView, ImageDecoder, ImageFormat};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

static CLIPBOARD_LOCK: Mutex<Option<Instant>> = Mutex::new(None);

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

fn encode_animated_webp_to_gif_bytes(
    bytes: &[u8],
) -> Result<Option<(Vec<u8>, u32, u32)>, String> {
    let decoder = WebPDecoder::new(Cursor::new(bytes))
        .map_err(|e| format!("Error leyendo imagen WebP: {}", e))?;
    if !decoder.has_animation() {
        return Ok(None);
    }

    let (w, h) = decoder.dimensions();
    let frames = decoder
        .into_frames()
        .collect_frames()
        .map_err(|e| format!("Error decodificando fotogramas WebP: {}", e))?;

    if frames.len() <= 1 {
        return Ok(None);
    }

    let normalized_frames = frames.into_iter().map(|frame| {
        let (numer, denom) = frame.delay().numer_denom_ms();
        let delay_ms = numer / denom.max(1);
        if delay_ms > 0 && delay_ms < 10 {
            Frame::from_parts(
                frame.into_buffer(),
                0,
                0,
                Delay::from_numer_denom_ms(10, 1),
            )
        } else {
            frame
        }
    });

    let mut gif_bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new_with_speed(&mut gif_bytes, 10);
        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(|e| format!("Error configurando animación GIF: {}", e))?;
        encoder
            .encode_frames(normalized_frames)
            .map_err(|e| format!("Error convirtiendo WebP animado a GIF: {}", e))?;
    }

    Ok(Some((gif_bytes, w, h)))
}

fn normalize_download_url(url: &str) -> String {
    let without_fragment = url.split('#').next().unwrap_or(url).trim();
    let Some((base, query)) = without_fragment.split_once('?') else {
        return without_fragment.to_string();
    };

    let kept_params: Vec<&str> = query
        .split('&')
        .filter(|param| {
            let key = param.split('=').next().unwrap_or("").trim();
            !key.is_empty() && !key.eq_ignore_ascii_case("size")
        })
        .collect();

    if kept_params.is_empty() {
        base.to_string()
    } else {
        format!("{}?{}", base, kept_params.join("&"))
    }
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

    if !matches!(
        guessed,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP
    ) {
        return Err(
            "Formato de archivo no compatible. Solo se admiten imágenes PNG, JPG, JPEG, GIF y WEBP."
                .to_string(),
        );
    }

    let mut is_animated_gif = false;
    let mut gif_dimensions: Option<(u32, u32)> = None;
    let mut converted_webp_gif: Option<(Vec<u8>, u32, u32)> = None;

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
    } else if guessed == ImageFormat::WebP {
        converted_webp_gif = encode_animated_webp_to_gif_bytes(bytes)?;
    }

    let target_ext = if is_animated_gif || converted_webp_gif.is_some() {
        "gif"
    } else {
        "webp"
    };
    let dest = abs_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), target_ext));

    let (w, h, format) = if is_animated_gif {
        fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        let (w, h) = gif_dimensions.unwrap_or_else(|| {
            image::load_from_memory(bytes)
                .map(|i| i.dimensions())
                .unwrap_or((0, 0))
        });
        (w as i32, h as i32, "gif".to_string())
    } else if let Some((gif_bytes, w, h)) = converted_webp_gif {
        fs::write(&dest, &gif_bytes).map_err(|e| e.to_string())?;
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

fn count_db_emotes(db_path: &Path) -> usize {
    if !db_path.exists() {
        return 0;
    }
    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row("SELECT COUNT(*) FROM emotes", [], |r| r.get::<_, i64>(0))
        .unwrap_or(0)
        .max(0) as usize
}

/// Copia los metadatos (nombre, tags, votos) de los emotes de `secondary_db` hacia `primary_db`
/// para los archivos que aún no estén en `primary_db` (o cuyos registros en `primary_db` no tengan tags/nombre propio).
fn merge_secondary_db_into_primary(secondary_db: &Path, primary_db: &Path, target_dir: &Path) {
    if !secondary_db.exists() || !primary_db.exists() || secondary_db == primary_db {
        return;
    }
    let sec_conn = match Connection::open(secondary_db) {
        Ok(c) => c,
        Err(_) => return,
    };
    let pri_conn = match Connection::open(primary_db) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Asegurar que las rutas en primary_db apunten a target_dir por nombre de archivo
    if let Ok(mut stmt) = pri_conn.prepare("SELECT id, path FROM emotes") {
        let pri_rows: Vec<(i64, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        for (id, raw_path) in pri_rows {
            let clean = clean_path_str(&raw_path);
            if let Some(fname) = Path::new(&clean).file_name() {
                let dest = normalize_path(&target_dir.join(fname));
                if dest != clean {
                    let _ = pri_conn.execute(
                        "UPDATE emotes SET path = ?1 WHERE id = ?2",
                        params![dest, id],
                    );
                }
            }
        }
    }

    // Mapear archivos existentes en primary_db por nombre de archivo en minúsculas
    let mut pri_by_fname: HashMap<String, (i64, String, Option<String>)> = HashMap::new();
    if let Ok(mut stmt) = pri_conn.prepare("SELECT id, name, tag, path FROM emotes") {
        let rows: Vec<(i64, Option<String>, Option<String>, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        for (id, name, tag, raw_path) in rows {
            if let Some(fname) = Path::new(&clean_path_str(&raw_path))
                .file_name()
                .and_then(|s| s.to_str())
            {
                pri_by_fname.insert(fname.to_lowercase(), (id, name.unwrap_or_default(), tag));
            }
        }
    }

    let query = "SELECT e.name, e.tag, e.path, e.original_width, e.original_height, e.format, e.created_at,
                        COALESCE(v.copy_count, e.copy_count, 0),
                        COALESCE(v.interaction_count, e.interaction_count, 0),
                        COALESCE(v.last_used_at, e.last_used_at)
                 FROM emotes e
                 LEFT JOIN emote_votes v ON v.emote_id = e.id";
    let mut stmt = match sec_conn.prepare(query) {
        Ok(s) => s,
        Err(_) => return,
    };

    let sec_rows: Vec<(
        Option<String>,
        Option<String>,
        String,
        i32,
        i32,
        String,
        String,
        i64,
        i64,
        Option<String>,
    )> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get::<_, Option<i32>>(3)?.unwrap_or(0),
                r.get::<_, Option<i32>>(4)?.unwrap_or(0),
                r.get::<_, Option<String>>(5)?.unwrap_or_else(|| "webp".to_string()),
                r.get::<_, Option<String>>(6)?.unwrap_or_default(),
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
            ))
        })
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    for (name, tag, raw_path, w, h, format, created_at, copies, interactions, last_used) in sec_rows
    {
        let clean = clean_path_str(&raw_path);
        let p = Path::new(&clean);
        let fname = match p.file_name().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };
        let stem = extract_stem(p);
        let dest_path = target_dir.join(fname);
        let dest_str = normalize_path(&dest_path);

        if let Some(&(pri_id, ref pri_name, ref pri_tag)) =
            pri_by_fname.get(&fname.to_lowercase())
        {
            // Si el registro primario tenía nombre autogenerado (igual al UUID) y el secundario tiene nombre propio, conservarlo
            if let Some(ref sec_name) = name {
                if !sec_name.is_empty() && (pri_name.is_empty() || pri_name == &stem) && sec_name != &stem {
                    let _ = pri_conn.execute(
                        "UPDATE emotes SET name = ?1 WHERE id = ?2",
                        params![sec_name, pri_id],
                    );
                }
            }
            // Si el registro primario no tenía tags y el secundario sí, conservarlos
            if pri_tag.as_deref().unwrap_or("").trim().is_empty() {
                if let Some(ref sec_tag) = tag {
                    if !sec_tag.trim().is_empty() {
                        let _ = pri_conn.execute(
                            "UPDATE emotes SET tag = ?1 WHERE id = ?2",
                            params![sec_tag, pri_id],
                        );
                    }
                }
            }
            if copies > 0 || interactions > 0 {
                let _ = pri_conn.execute(
                    "INSERT INTO emote_votes (emote_id, copy_count, interaction_count, score, last_used_at)
                     VALUES (?1, ?2, ?3, ?2, ?4)
                     ON CONFLICT(emote_id) DO UPDATE SET
                         copy_count = MAX(copy_count, excluded.copy_count),
                         interaction_count = MAX(interaction_count, excluded.interaction_count),
                         score = MAX(copy_count, excluded.copy_count),
                         last_used_at = COALESCE(last_used_at, excluded.last_used_at)",
                    params![pri_id, copies, interactions, last_used],
                );
            }
        } else {
            // Insertar emote que venía de la otra carpeta con todos sus metadatos intactos
            if pri_conn
                .execute(
                    "INSERT INTO emotes (name, tag, path, original_width, original_height, format, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        name.unwrap_or_else(|| stem),
                        tag,
                        dest_str,
                        w,
                        h,
                        format,
                        created_at
                    ],
                )
                .is_ok()
            {
                let new_id = pri_conn.last_insert_rowid();
                if copies > 0 || interactions > 0 {
                    let _ = pri_conn.execute(
                        "INSERT OR REPLACE INTO emote_votes (emote_id, copy_count, interaction_count, score, last_used_at)
                         VALUES (?1, ?2, ?3, ?2, ?4)",
                        params![new_id, copies, interactions, last_used],
                    );
                }
            }
        }
    }
}

#[tauri::command]
pub fn set_storage_path(
    path: String,
    migrate: Option<bool>,
    overwrite_db: Option<bool>,
) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(|e| format!("No se pudo crear la carpeta: {}", e))?;
    }
    let abs = p.canonicalize().unwrap_or(p);
    let abs_str = normalize_path(&abs);

    if abs_str.contains("src-tauri") {
        return Err(
            "No se puede ubicar la carpeta de emotes y base de datos dentro de src-tauri"
                .to_string(),
        );
    }

    let should_migrate = migrate.unwrap_or(false);
    let cur_db_dir = get_active_db_dir();
    let cur_db_abs = cur_db_dir.canonicalize().unwrap_or(cur_db_dir);
    upgrade_legacy_db_in_dir(&cur_db_abs);
    upgrade_legacy_db_in_dir(&abs);
    let old_db = cur_db_abs.join(DB_FILE_NAME);
    let new_db = abs.join(DB_FILE_NAME);

    let old_count = count_db_emotes(&old_db);
    let new_count = count_db_emotes(&new_db);

    if should_migrate
        && old_db != new_db
        && old_count > 0
        && new_count > 0
        && overwrite_db.is_none()
    {
        return Err(format!("DB_EXISTS_IN_TARGET:{}:{}", old_count, new_count));
    }

    let replace_target_db = match overwrite_db {
        Some(v) => v,
        None => {
            if old_count == 0 && new_count > 0 {
                false
            } else {
                true
            }
        }
    };

    if should_migrate {
        let conn = ensure_db()?;
        let old_dir = get_active_emotes_dir(&conn);
        let old_abs = old_dir.canonicalize().unwrap_or(old_dir);
        let old_str = normalize_path(&old_abs);

        if old_str.to_lowercase() != abs_str.to_lowercase() {
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
                        if !dest_path.exists() || replace_target_db {
                            if fs::copy(src_path, &dest_path).is_ok() {
                                let _ = fs::remove_file(src_path);
                            }
                        } else {
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
        drop(conn);

        if old_db != new_db && old_db.exists() && new_db.exists() {
            if replace_target_db {
                merge_secondary_db_into_primary(&new_db, &old_db, &abs);
            } else {
                merge_secondary_db_into_primary(&old_db, &new_db, &abs);
            }
        }
    }

    let _ = set_active_db_dir(&abs_str, should_migrate && replace_target_db)?;

    if should_migrate && !replace_target_db && old_db != new_db && old_db.exists() {
        let _ = fs::remove_file(&old_db);
    }

    if !should_migrate || !replace_target_db {
        let conn = ensure_db()?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('storage_path', ?1)",
            params![&abs_str],
        )
        .map_err(|e| e.to_string())?;
    }

    if should_migrate {
        let _ = resync_emotes();
    }

    Ok(abs_str)
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

    // Detectar si es un GIF animado (>1 fotograma) o WebP animado para conservar su secuencia como GIF
    let mut is_animated_gif = false;
    let mut gif_dimensions: Option<(u32, u32)> = None;
    let mut converted_webp_gif: Option<(Vec<u8>, u32, u32)> = None;
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
    } else if orig_ext == "webp" {
        if let Ok(bytes) = fs::read(src) {
            converted_webp_gif = encode_animated_webp_to_gif_bytes(&bytes)?;
        }
    }

    let target_ext = if is_animated_gif || converted_webp_gif.is_some() {
        "gif"
    } else {
        "webp"
    };
    let dest = abs_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), target_ext));

    let (w, h, format) = if is_animated_gif {
        fs::copy(src, &dest).map_err(|e| e.to_string())?;
        let (w, h) = gif_dimensions.unwrap_or_else(|| {
            image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0))
        });
        (w as i32, h as i32, "gif".to_string())
    } else if let Some((gif_bytes, w, h)) = converted_webp_gif {
        fs::write(&dest, &gif_bytes).map_err(|e| e.to_string())?;
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

fn download_url_bytes(url: &str) -> Result<Vec<u8>, String> {
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
        url,
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

    Ok(output.stdout)
}

#[tauri::command]
pub fn import_from_url(url: String, name: Option<String>) -> Result<Emote, String> {
    let clean_url = url.trim();
    if !clean_url.starts_with("http://") && !clean_url.starts_with("https://") {
        return Err("La URL debe comenzar con http:// o https://".to_string());
    }

    let url_without_query = clean_url.split('?').next().unwrap_or(clean_url);
    if let Some(idx) = url_without_query.rfind('.') {
        let ext = &url_without_query[idx + 1..];
        if !ext.is_empty() && ext.len() <= 5 && !ext.contains('/') {
            let ext_lower = ext.to_lowercase();
            let known_non_images = [
                "html", "htm", "php", "asp", "aspx", "js", "css", "pdf", "zip", "rar", "exe",
                "mp4", "mp3", "json", "xml", "txt",
            ];
            if known_non_images.contains(&ext_lower.as_str()) {
                return Err(format!(
                    "La URL apunta a un archivo '{}' no compatible. Solo se admiten imágenes PNG, JPG, JPEG, GIF y WEBP.",
                    ext_lower
                ));
            }
        }
    }

    let normalized_url = normalize_download_url(clean_url);
    let bytes = match download_url_bytes(&normalized_url) {
        Ok(b) if !b.is_empty() => b,
        Err(_) | Ok(_) if normalized_url != clean_url => download_url_bytes(clean_url)?,
        Err(e) => return Err(e),
        Ok(b) => b,
    };

    let emote_name = name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| infer_name_from_url(clean_url));

    save_raw_bytes_as_emote(&bytes, &emote_name)
}

#[tauri::command]
pub fn import_from_clipboard(name: Option<String>) -> Result<Vec<Emote>, String> {
    {
        let mut lock = CLIPBOARD_LOCK.lock().map_err(|e| e.to_string())?;
        let now = Instant::now();
        if let Some(prev) = *lock {
            if now.duration_since(prev).as_millis() < 800 {
                return Err("Una importación del portapapeles ya está en curso".to_string());
            }
        }
        *lock = Some(now);
    }

    let custom_name = name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    match clipboard::read_clipboard_payload()? {
        ClipboardPayload::LocalPaths(paths) => {
            let conn = ensure_db()?;
            let dir = get_active_emotes_dir(&conn);
            if !dir.exists() {
                fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            }
            let abs_dir = dir.canonicalize().unwrap_or(dir);
            let mut created = Vec::new();
            let mut had_unsupported = false;

            for p in paths {
                if !p.exists() {
                    continue;
                }
                if p.is_dir() {
                    let entries = fs::read_dir(&p).map_err(|e| e.to_string())?;
                    for entry in entries.flatten() {
                        let ep = entry.path();
                        if ep.is_file() {
                            if let Some(emote) = import_single_file(&conn, &abs_dir, &ep)? {
                                created.push(emote);
                            }
                        }
                    }
                } else if p.is_file() {
                    let ext = p
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
                        if let Some(emote) = import_single_file(&conn, &abs_dir, &p)? {
                            created.push(emote);
                        }
                    } else {
                        had_unsupported = true;
                    }
                }
            }

            if created.is_empty() {
                if had_unsupported {
                    return Err(
                        "El archivo no es compatible. Solo se admiten archivos .png, .jpg, .jpeg, .gif y .webp"
                            .to_string(),
                    );
                } else {
                    return Err(
                        "No se encontraron imágenes compatibles (.png, .jpg, .jpeg, .gif, .webp) en el directorio o ruta indicada"
                            .to_string(),
                    );
                }
            }
            Ok(created)
        }
        ClipboardPayload::RawImageBytes(bytes) => {
            let final_name = custom_name.unwrap_or_else(|| {
                format!("clipboard_{}", &uuid::Uuid::new_v4().to_string()[..6])
            });
            let emote = save_raw_bytes_as_emote(&bytes, &final_name)?;
            Ok(vec![emote])
        }
        #[cfg(windows)]
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
            let emote = save_raw_bytes_as_emote(&buf.into_inner(), &final_name)?;
            Ok(vec![emote])
        }
        ClipboardPayload::TextUrl(url) => {
            let emote = import_from_url(url, custom_name)?;
            Ok(vec![emote])
        }
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
            let clean_path = Path::new(&clean);
            if clean_path.exists() {
                existing_db.insert(clean.to_lowercase(), id);
            } else if let Some(fname) = clean_path.file_name() {
                let relocated = abs_dir.join(fname);
                if relocated.exists() {
                    let relocated_str = normalize_path(&relocated);
                    let _ = conn.execute(
                        "UPDATE emotes SET path = ?1 WHERE id = ?2",
                        params![&relocated_str, id],
                    );
                    existing_db.insert(relocated_str.to_lowercase(), id);
                } else {
                    let _ = conn.execute("DELETE FROM emote_votes WHERE emote_id = ?1", params![id]);
                    let _ = conn.execute("DELETE FROM emotes WHERE id = ?1", params![id]);
                    stats.removed_missing_from_db += 1;
                }
            } else {
                let _ = conn.execute("DELETE FROM emote_votes WHERE emote_id = ?1", params![id]);
                let _ = conn.execute("DELETE FROM emotes WHERE id = ?1", params![id]);
                stats.removed_missing_from_db += 1;
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
        } else if ext == "webp" {
            let mut converted_to_gif = false;
            if let Ok(bytes) = fs::read(&entry_path) {
                if let Ok(Some((gif_bytes, w, h))) = encode_animated_webp_to_gif_bytes(&bytes) {
                    let mut dest_path = entry_path.with_extension("gif");
                    if dest_path.exists() && dest_path != entry_path {
                        dest_path = abs_dir.join(format!(
                            "{}_{}.gif",
                            extract_stem(&entry_path),
                            &uuid::Uuid::new_v4().to_string()[..8]
                        ));
                    }
                    if fs::write(&dest_path, &gif_bytes).is_ok() {
                        converted_to_gif = true;
                        let new_path_str = normalize_path(&dest_path);
                        if dest_path != entry_path {
                            let _ = fs::remove_file(&entry_path);
                        }
                        if let Some(&id) = existing_db.get(&orig_key) {
                            let _ = conn.execute(
                                "UPDATE emotes SET path = ?1, format = 'gif', original_width = ?2, original_height = ?3 WHERE id = ?4",
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
                                "gif",
                                &now,
                            );
                            stats.added_to_db += 1;
                        }
                    }
                }
            }
            if !converted_to_gif && !existing_db.contains_key(&orig_key) {
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

#[tauri::command]
pub fn create_backup(destination_path: Option<String>) -> Result<BackupResult, String> {
    let dest_clean = destination_path.as_deref().map(clean_path_str);
    let dest_path = dest_clean.as_deref().map(Path::new);
    let info = db::create_db_backup(dest_path)?;
    Ok(BackupResult {
        success: true,
        message: format!(
            "Copia de seguridad guardada exitosamente: {} ({} emotes)",
            info.file_name, info.emote_count
        ),
        backup: info,
    })
}

#[tauri::command]
pub fn restore_backup(backup_path: String) -> Result<RestoreResult, String> {
    let clean = clean_path_str(&backup_path);
    db::restore_db_backup(Path::new(&clean))
}

#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupInfo>, String> {
    db::list_db_backups()
}

#[tauri::command]
pub fn inspect_backup(backup_path: String) -> Result<BackupInfo, String> {
    let clean = clean_path_str(&backup_path);
    db::inspect_backup_file(Path::new(&clean))
}

#[tauri::command]
pub fn delete_backup(backup_path: String) -> Result<bool, String> {
    let clean = clean_path_str(&backup_path);
    db::delete_db_backup(Path::new(&clean))
}

#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    Ok(db::CURRENT_APP_VERSION.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_stem() {
        assert_eq!(extract_stem(Path::new("/tmp/pepe_dance.gif")), "pepe_dance");
        assert_eq!(extract_stem(Path::new("cat.webp")), "cat");
    }

    #[test]
    fn test_infer_name_from_url() {
        assert_eq!(
            infer_name_from_url("https://cdn.example.com/emotes/pogchamp.png?v=123#main"),
            "pogchamp"
        );
        assert_eq!(
            infer_name_from_url("https://cdn.example.com/"),
            "cdn.example"
        );
    }

    #[test]
    fn test_insert_emote_record_in_memory() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE emotes (
                id INTEGER PRIMARY KEY,
                name TEXT,
                tag TEXT,
                path TEXT NOT NULL,
                original_width INTEGER,
                original_height INTEGER,
                format TEXT,
                created_at TEXT
            );",
        )
        .unwrap();

        let id = insert_emote_record(
            &conn,
            "pepe",
            "/tmp/pepe.webp",
            128,
            128,
            "webp",
            "2026-11-01T00:00:00Z",
        )
        .expect("insert emote");
        assert_eq!(id, 1);
    }

    #[test]
    fn test_normalize_download_url_removes_default_size() {
        assert_eq!(
            normalize_download_url(
                "https://cdn.discordapp.com/emojis/1539337442191212695.webp?size=32&animated=true"
            ),
            "https://cdn.discordapp.com/emojis/1539337442191212695.webp?animated=true"
        );
        assert_eq!(
            normalize_download_url(
                "https://cdn.discordapp.com/emojis/1539337442191212695.webp?size=32"
            ),
            "https://cdn.discordapp.com/emojis/1539337442191212695.webp"
        );
        assert_eq!(
            normalize_download_url(
                "https://cdn.discordapp.com/emojis/123.webp?quality=lossless&size=48&animated=true"
            ),
            "https://cdn.discordapp.com/emojis/123.webp?quality=lossless&animated=true"
        );
        assert_eq!(
            normalize_download_url("https://example.com/emote.webp?v=1"),
            "https://example.com/emote.webp?v=1"
        );
    }

    fn build_minimal_animated_webp(width: u32, height: u32) -> Vec<u8> {
        let mut frame1_webp = Cursor::new(Vec::new());
        let img1 = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
        image::DynamicImage::ImageRgba8(img1)
            .write_to(&mut frame1_webp, ImageFormat::WebP)
            .unwrap();
        let vp8l_1 = &frame1_webp.into_inner()[12..];

        let mut frame2_webp = Cursor::new(Vec::new());
        let img2 = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 255, 0, 255]));
        image::DynamicImage::ImageRgba8(img2)
            .write_to(&mut frame2_webp, ImageFormat::WebP)
            .unwrap();
        let vp8l_2 = &frame2_webp.into_inner()[12..];

        let make_anmf = |subchunk: &[u8]| -> Vec<u8> {
            let payload_len = 16 + subchunk.len();
            let mut chunk = Vec::with_capacity(8 + payload_len + (payload_len % 2));
            chunk.extend_from_slice(b"ANMF");
            chunk.extend_from_slice(&(payload_len as u32).to_le_bytes());
            // X (3b), Y (3b)
            chunk.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
            // w - 1 (3b), h - 1 (3b)
            let w_m1 = (width - 1).to_le_bytes();
            let h_m1 = (height - 1).to_le_bytes();
            chunk.extend_from_slice(&[w_m1[0], w_m1[1], w_m1[2], h_m1[0], h_m1[1], h_m1[2]]);
            // duration 50ms (3b) + flags (1b: disposal=0, blend=1 -> 0x02)
            chunk.extend_from_slice(&[50, 0, 0, 0x02]);
            chunk.extend_from_slice(subchunk);
            if payload_len % 2 == 1 {
                chunk.push(0);
            }
            chunk
        };

        let anmf1 = make_anmf(vp8l_1);
        let anmf2 = make_anmf(vp8l_2);

        let mut body = Vec::new();
        body.extend_from_slice(b"WEBP");
        // VP8X chunk (10 bytes payload)
        body.extend_from_slice(b"VP8X");
        body.extend_from_slice(&10u32.to_le_bytes());
        // flags: animation (0x02) | alpha (0x10) = 0x12
        body.extend_from_slice(&[0x12, 0, 0, 0]);
        let w_m1 = (width - 1).to_le_bytes();
        let h_m1 = (height - 1).to_le_bytes();
        body.extend_from_slice(&[w_m1[0], w_m1[1], w_m1[2], h_m1[0], h_m1[1], h_m1[2]]);

        // ANIM chunk (6 bytes payload)
        body.extend_from_slice(b"ANIM");
        body.extend_from_slice(&6u32.to_le_bytes());
        body.extend_from_slice(&[0, 0, 0, 0, 0, 0]);

        body.extend_from_slice(&anmf1);
        body.extend_from_slice(&anmf2);

        let mut riff = Vec::with_capacity(8 + body.len());
        riff.extend_from_slice(b"RIFF");
        riff.extend_from_slice(&(body.len() as u32).to_le_bytes());
        riff.extend_from_slice(&body);
        riff
    }

    #[test]
    fn test_encode_animated_webp_to_gif_bytes() {
        let anim_webp = build_minimal_animated_webp(16, 16);
        let result = encode_animated_webp_to_gif_bytes(&anim_webp).expect("should decode webp");
        let (gif_bytes, w, h) = result.expect("should detect animated webp");
        assert_eq!((w, h), (16, 16));
        assert_eq!(image::guess_format(&gif_bytes).unwrap(), ImageFormat::Gif);

        let decoder = GifDecoder::new(Cursor::new(&gif_bytes)).expect("valid gif");
        let frames = decoder.into_frames().collect_frames().expect("gif frames");
        assert_eq!(frames.len(), 2);

        // Static WebP should return Ok(None)
        let mut static_webp = Cursor::new(Vec::new());
        let static_img = image::RgbaImage::from_pixel(8, 8, image::Rgba([0, 0, 255, 255]));
        image::DynamicImage::ImageRgba8(static_img)
            .write_to(&mut static_webp, ImageFormat::WebP)
            .unwrap();
        let static_res =
            encode_animated_webp_to_gif_bytes(&static_webp.into_inner()).expect("valid static webp");
        assert!(static_res.is_none());
    }
}
