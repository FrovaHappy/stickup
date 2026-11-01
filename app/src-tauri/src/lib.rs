// Tauri command implementations for Emote management
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{AnimationDecoder, Frame, GenericImageView, ImageFormat};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Serialize)]
pub struct Emote {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
    pub path: String,
    pub original_width: i32,
    pub original_height: i32,
    pub format: String,
    pub created_at: String,
    pub file_size: Option<u64>,
}

fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s
    }
}

fn db_path() -> PathBuf {
    if let Ok(current) = std::env::current_dir() {
        if current.ends_with("src-tauri") {
            if let Some(parent) = current.parent() {
                return parent.join("emotes.db");
            }
        }
        return current.join("emotes.db");
    }
    PathBuf::from("emotes.db")
}

fn get_default_emotes_dir() -> PathBuf {
    if let Ok(current) = std::env::current_dir() {
        if current.ends_with("src-tauri") {
            if let Some(parent) = current.parent() {
                return parent.join("emotes");
            }
        }
        return current.join("emotes");
    }
    PathBuf::from("emotes")
}

fn get_active_emotes_dir(conn: &Connection) -> PathBuf {
    if let Ok(mut stmt) = conn.prepare("SELECT value FROM settings WHERE key = 'storage_path'") {
        if let Ok(path_str) = stmt.query_row([], |row| row.get::<_, String>(0)) {
            let clean = if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                path_str
            };
            if !clean.contains("src-tauri") {
                let p = PathBuf::from(clean);
                if !p.as_os_str().is_empty() {
                    return p;
                }
            }
        }
    }
    let default_dir = get_default_emotes_dir();
    let norm = normalize_path(&default_dir);
    let _ = conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('storage_path', ?1)",
        params![norm],
    );
    default_dir
}

fn ensure_db() -> Result<Connection, String> {
    let db = db_path();
    if let Some(parent) = db.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let conn = Connection::open(&db).map_err(|e| format!("{}: {}", e, db.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS emotes (
            id INTEGER PRIMARY KEY,
            name TEXT,
            tag TEXT,
            path TEXT NOT NULL,
            original_width INTEGER,
            original_height INTEGER,
            format TEXT,
            created_at TEXT
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );",
    )
    .map_err(|e| e.to_string())?;

    let _ = conn.execute(
        "UPDATE emotes SET path = SUBSTR(path, 5) WHERE path LIKE '\\\\?\\%'",
        [],
    );

    Ok(conn)
}

#[tauri::command]
fn list_emotes() -> Result<Vec<Emote>, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare("SELECT id, name, tag, path, original_width, original_height, format, created_at FROM emotes ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let emote_iter = stmt
        .query_map([], |row| {
            let raw_path: String = row.get(3)?;
            let clean_path = if let Some(stripped) = raw_path.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                raw_path
            };
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
fn get_storage_path() -> Result<String, String> {
    let conn = ensure_db()?;
    let dir = get_active_emotes_dir(&conn);
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    let abs = dir.canonicalize().unwrap_or(dir);
    Ok(normalize_path(&abs))
}

#[tauri::command]
fn set_storage_path(path: String) -> Result<String, String> {
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
fn import_folder(folder_path: String) -> Result<Vec<Emote>, String> {
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

    // Al importar una carpeta, migra automáticamente todas las imágenes a WebP
    import_emotes_internal(paths, true)
}

#[tauri::command]
fn import_emotes(paths: Vec<String>) -> Result<Vec<Emote>, String> {
    import_emotes_internal(paths, false)
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

        // Detectar si es un GIF animado: si tiene más de 1 fotograma, debe conservarse como GIF
        // para no perder la secuencia de fotogramas (WebP estático perdería la animación).
        let mut is_animated_gif = false;
        let mut gif_dimensions: Option<(u32, u32)> = None;
        if orig_ext == "gif" {
            if let Ok(file) = fs::File::open(&src) {
                let reader = std::io::BufReader::new(file);
                if let Ok(decoder) = GifDecoder::new(reader) {
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
            // Copiar directamente el GIF animado para preservar todos los fotogramas y tiempos
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            let (w, h) = gif_dimensions.unwrap_or_else(|| {
                image::open(&dest).map(|i| i.dimensions()).unwrap_or((0, 0))
            });
            (w as i32, h as i32, "gif".to_string())
        } else if convert_to_webp && orig_ext != "webp" {
            // Cargar imagen de origen y guardarla directamente en formato WebP
            let img = image::open(&src).map_err(|e| format!("Error abriendo {:?}: {}", src, e))?;
            let (w, h) = img.dimensions();
            let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
            img.write_to(&mut file, ImageFormat::WebP)
                .map_err(|e| format!("Error convirtiendo a WebP: {}", e))?;
            (w as i32, h as i32, "webp".to_string())
        } else {
            // Ya es webp o no requiere conversión: copia directa
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
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
fn delete_emote(id: i64) -> Result<bool, String> {
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
fn update_tag(id: i64, name: Option<String>, tag: Option<String>) -> Result<bool, String> {
    let conn = ensure_db()?;
    if let Some(n) = name {
        if let Some(t) = tag {
            conn.execute(
                "UPDATE emotes SET name = ?1, tag = ?2 WHERE id = ?3",
                params![n, t, id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "UPDATE emotes SET name = ?1 WHERE id = ?2",
                params![n, id],
            )
            .map_err(|e| e.to_string())?;
        }
    } else if let Some(t) = tag {
        conn.execute(
            "UPDATE emotes SET tag = ?1 WHERE id = ?2",
            params![t, id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

#[tauri::command]
fn resize_emote(id: i64, width: u32, height: u32) -> Result<bool, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare("SELECT path FROM emotes WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let path: String = stmt
        .query_row([id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let img = image::open(&path).map_err(|e| e.to_string())?;
    let resized = image::imageops::resize(&img, width, height, FilterType::Lanczos3);
    resized.save(&path).map_err(|e| e.to_string())?;

    // Update DB dimensions
    conn.execute(
        "UPDATE emotes SET original_width = ?1, original_height = ?2 WHERE id = ?3",
        params![width as i32, height as i32, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn convert_emote(id: i64, new_format: &str) -> Result<String, String> {
    let conn = ensure_db()?;
    let mut stmt = conn
        .prepare("SELECT path FROM emotes WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let path: String = stmt
        .query_row([id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let img = image::open(&path).map_err(|e| e.to_string())?;
    let dest_path = Path::new(&path).with_extension(new_format);
    let fmt = match new_format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::WebP,
        _ => ImageFormat::Png,
    };
    img.write_to(
        &mut fs::File::create(&dest_path).map_err(|e| e.to_string())?,
        fmt,
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE emotes SET path = ?1, format = ?2 WHERE id = ?3",
        params![dest_path.to_string_lossy(), new_format, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(dest_path.to_string_lossy().to_string())
}

#[cfg(target_os = "linux")]
mod clipboard_linux {
    use std::fs;
    use std::process::Command;

    pub fn copy_file_and_image_to_clipboard(
        file_path: &str,
        raw_bytes: Option<(&[u8], &str)>,
        _is_animated_gif: bool,
    ) -> Result<(), String> {
        let uri = format!("file://{}\r\n", file_path);

        // 1. Probar Wayland wl-copy si está disponible
        if let Ok(mut child) = Command::new("wl-copy")
            .arg("--type")
            .arg("text/uri-list")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(uri.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // 2. Probar X11 xclip si está disponible
        let mime_type = match raw_bytes {
            Some((_, "gif")) => "image/gif",
            Some((_, "webp")) => "image/webp",
            _ => "image/png",
        };

        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard", "-target", mime_type])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                if let Some((bytes, _)) = raw_bytes {
                    let _ = stdin.write_all(bytes);
                } else if let Ok(data) = fs::read(file_path) {
                    let _ = stdin.write_all(&data);
                }
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // 3. Fallback xclip text/uri-list
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard", "-target", "text/uri-list"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(uri.as_bytes());
            }
            let _ = child.wait();
        }

        Ok(())
    }
}

#[cfg(target_os = "android")]
mod clipboard_android {
    pub fn copy_file_and_image_to_clipboard(
        _file_path: &str,
        _raw_bytes: Option<(&[u8], &str)>,
        _is_animated_gif: bool,
    ) -> Result<(), String> {
        // En Android, el intercambio de stickers/emotes se delega principalmente a Share Intent o FileProvider
        Ok(())
    }
}

#[cfg(windows)]
mod clipboard_win {
    use std::ffi::CString;
    use std::mem::size_of;
    use std::ptr;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatA, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GHND, GMEM_MOVEABLE,
    };
    use windows_sys::Win32::UI::Shell::DROPFILES;

    const CF_DIB: u32 = 8;
    const CF_HDROP: u32 = 15;

    pub fn copy_file_and_image_to_clipboard(
        file_path: &str,
        raw_bytes: Option<(&[u8], &str)>, // (image_bytes, format_str)
        img_rgba: Option<(&[u8], u32, u32)>, // (rgba_pixels, width, height)
        is_animated_gif: bool,
    ) -> Result<(), String> {
        unsafe {
            if OpenClipboard(0 as HWND) == 0 {
                return Err("No se pudo abrir el portapapeles de Windows".to_string());
            }
            EmptyClipboard();

            // 1. CF_HDROP: Permite pegar el archivo directamente en Discord, Telegram, Explorador de Windows, etc.
            let mut wide_chars: Vec<u16> = file_path.encode_utf16().collect();
            wide_chars.push(0); // null terminator for string
            wide_chars.push(0); // double null terminator for DROPFILES list

            let dropfiles_size = size_of::<DROPFILES>();
            let total_size = dropfiles_size + wide_chars.len() * 2;

            let h_drop = GlobalAlloc(GHND, total_size);
            if !h_drop.is_null() {
                let p_drop = GlobalLock(h_drop) as *mut u8;
                if !p_drop.is_null() {
                    let drop = p_drop as *mut DROPFILES;
                    (*drop).pFiles = dropfiles_size as u32;
                    (*drop).pt.x = 0;
                    (*drop).pt.y = 0;
                    (*drop).fNC = 0;
                    (*drop).fWide = 1; // Unicode UTF-16

                    ptr::copy_nonoverlapping(
                        wide_chars.as_ptr() as *const u8,
                        p_drop.add(dropfiles_size),
                        wide_chars.len() * 2,
                    );
                    GlobalUnlock(h_drop);
                    SetClipboardData(CF_HDROP, h_drop as _);
                }
            }

            // 2. Formato específico registrado (e.g. "image/gif", "GIF", "image/webp", "PNG")
            if let Some((bytes, fmt_name)) = raw_bytes {
                let format_names = match fmt_name.to_lowercase().as_str() {
                    "gif" => vec!["image/gif", "GIF"],
                    "png" => vec!["PNG", "image/png"],
                    "webp" => vec!["image/webp", "WEBP"],
                    "jpg" | "jpeg" => vec!["image/jpeg", "image/jpg", "JFIF"],
                    _ => vec![],
                };

                for format_name in format_names {
                    if let Ok(c_str) = CString::new(format_name) {
                        let cf = RegisterClipboardFormatA(c_str.as_ptr() as *const u8);
                        if cf != 0 {
                            let h_custom = GlobalAlloc(GMEM_MOVEABLE, bytes.len());
                            if !h_custom.is_null() {
                                let p_custom = GlobalLock(h_custom) as *mut u8;
                                if !p_custom.is_null() {
                                    ptr::copy_nonoverlapping(bytes.as_ptr(), p_custom, bytes.len());
                                    GlobalUnlock(h_custom);
                                    SetClipboardData(cf, h_custom as _);
                                }
                            }
                        }
                    }
                }
            }

            // 3. CF_DIB: Formato universal de bitmap plano para aplicaciones como Paint, Photoshop, Word.
            // NOTA CRÍTICA: Si es un GIF animado, NO ponemos CF_DIB. Si se pone CF_DIB, Discord y Telegram
            // priorizan el DIB estático antes que el archivo CF_HDROP, congelando la animación en el primer frame.
            if !is_animated_gif {
                if let Some((rgba, width, height)) = img_rgba {
                    let row_stride = ((width * 32 + 31) / 32) * 4;
                    let image_size = (row_stride * height) as usize;
                    let header_size = size_of::<BITMAPINFOHEADER>();
                    let total_dib_size = header_size + image_size;

                    let h_dib = GlobalAlloc(GMEM_MOVEABLE, total_dib_size);
                    if !h_dib.is_null() {
                        let p_dib = GlobalLock(h_dib) as *mut u8;
                        if !p_dib.is_null() {
                            let bmi_header = p_dib as *mut BITMAPINFOHEADER;
                            (*bmi_header).biSize = header_size as u32;
                            (*bmi_header).biWidth = width as i32;
                            (*bmi_header).biHeight = height as i32; // bottom-up DIB
                            (*bmi_header).biPlanes = 1;
                            (*bmi_header).biBitCount = 32;
                            (*bmi_header).biCompression = BI_RGB;
                            (*bmi_header).biSizeImage = image_size as u32;
                            (*bmi_header).biXPelsPerMeter = 0;
                            (*bmi_header).biYPelsPerMeter = 0;
                            (*bmi_header).biClrUsed = 0;
                            (*bmi_header).biClrImportant = 0;

                            let pixel_dest = p_dib.add(header_size);
                            // Windows DIB es bottom-up (de abajo hacia arriba) y formato BGRA
                            for y in 0..height {
                                let src_y = height - 1 - y;
                                let src_row_start = (src_y * width * 4) as usize;
                                let dest_row_start = (y * row_stride) as usize;

                                for x in 0..width {
                                    let src_px = src_row_start + (x * 4) as usize;
                                    let dest_px = dest_row_start + (x * 4) as usize;
                                    let r = rgba[src_px];
                                    let g = rgba[src_px + 1];
                                    let b = rgba[src_px + 2];
                                    let a = rgba[src_px + 3];

                                    *pixel_dest.add(dest_px) = b;
                                    *pixel_dest.add(dest_px + 1) = g;
                                    *pixel_dest.add(dest_px + 2) = r;
                                    *pixel_dest.add(dest_px + 3) = a;
                                }
                            }

                            GlobalUnlock(h_dib);
                            SetClipboardData(CF_DIB, h_dib as _);
                        }
                    }
                }
            }

            CloseClipboard();
            Ok(())
        }
    }
}

#[tauri::command]
fn copy_to_clipboard(
    path: String,
    format: String,
    target_width: u32,
    target_height: u32,
) -> Result<bool, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("El archivo del emote no existe".to_string());
    }

    let orig_ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let req_format = format.to_lowercase();

    // Comprobar si se requieren transformaciones (redimensión o cambio de formato)
    let needs_resize = target_width > 0 && target_height > 0;

    let is_animated_gif = orig_ext == "gif" && req_format == "gif";
    let final_path: PathBuf;
    let raw_bytes: Vec<u8>;
    let final_format: String;
    let mut rgba_data: Option<(Vec<u8>, u32, u32)> = None;

    if is_animated_gif && !needs_resize {
        // GIF original intacto sin redimensionar
        final_path = p.to_path_buf();
        raw_bytes = fs::read(&p).map_err(|e| e.to_string())?;
        final_format = "gif".to_string();
    } else if is_animated_gif && needs_resize {
        // Redimensionar cada fotograma del GIF animado para preservar toda la secuencia
        let file = fs::File::open(&p).map_err(|e| e.to_string())?;
        let reader = std::io::BufReader::new(file);
        let decoder = GifDecoder::new(reader).map_err(|e| e.to_string())?;
        let frames = decoder.into_frames().collect_frames().map_err(|e| e.to_string())?;

        let mut resized_frames = Vec::new();
        for frame in frames {
            let delay = frame.delay();
            let img = image::DynamicImage::ImageRgba8(frame.into_buffer());
            let resized = image::imageops::resize(&img, target_width, target_height, FilterType::Lanczos3);
            resized_frames.push(Frame::from_parts(resized, 0, 0, delay));
        }

        let temp_dir = std::env::temp_dir().join("stickup_clipboard");
        let _ = fs::create_dir_all(&temp_dir);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("emote");
        let temp_file = temp_dir.join(format!("{}_{}x{}.gif", stem, target_width, target_height));
        
        let out_file = fs::File::create(&temp_file).map_err(|e| e.to_string())?;
        let mut encoder = GifEncoder::new(out_file);
        encoder.set_repeat(Repeat::Infinite).map_err(|e| e.to_string())?;
        encoder.encode_frames(resized_frames.into_iter()).map_err(|e| e.to_string())?;

        raw_bytes = fs::read(&temp_file).map_err(|e| e.to_string())?;
        final_path = temp_file;
        final_format = "gif".to_string();
    } else {
        // Procesar imagen estática (PNG, WebP, JPG) con el formato seleccionado
        let img = image::open(&p).map_err(|e| format!("Error cargando imagen: {}", e))?;
        let processed = if needs_resize
            && (img.width() != target_width || img.height() != target_height)
        {
            image::imageops::resize(&img, target_width, target_height, FilterType::Lanczos3)
        } else {
            img.to_rgba8()
        };

        let (w, h) = (processed.width(), processed.height());
        let rgba_raw = processed.clone().into_raw();
        rgba_data = Some((rgba_raw, w, h));

        final_format = req_format.clone();
        let image_fmt = match req_format.as_str() {
            "webp" => ImageFormat::WebP,
            "gif" => ImageFormat::Gif,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            _ => ImageFormat::Png,
        };

        let mut buf = std::io::Cursor::new(Vec::new());
        processed
            .write_to(&mut buf, image_fmt)
            .map_err(|e| format!("Error codificando imagen: {}", e))?;
        raw_bytes = buf.into_inner();

        // Si el archivo original ya es exactamente del formato y tamaño solicitado, usamos su ruta
        if orig_ext == req_format && !needs_resize {
            final_path = p.to_path_buf();
        } else {
            // Guardar copia temporal para que el portapapeles (CF_HDROP) tenga un archivo físico
            let temp_dir = std::env::temp_dir().join("stickup_clipboard");
            let _ = fs::create_dir_all(&temp_dir);
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("emote");
            let temp_file = temp_dir.join(format!("{}_{}x{}.{}", stem, w, h, req_format));
            fs::write(&temp_file, &raw_bytes).map_err(|e| e.to_string())?;
            final_path = temp_file;
        }
    }

    let path_str = final_path.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        let rgba_ref = rgba_data
            .as_ref()
            .map(|(bytes, w, h)| (bytes.as_slice(), *w, *h));
        clipboard_win::copy_file_and_image_to_clipboard(
            &path_str,
            Some((&raw_bytes, &final_format)),
            rgba_ref,
            is_animated_gif,
        )?;
        Ok(true)
    }

    #[cfg(target_os = "linux")]
    {
        clipboard_linux::copy_file_and_image_to_clipboard(
            &path_str,
            Some((&raw_bytes, &final_format)),
            is_animated_gif,
        )?;
        Ok(true)
    }

    #[cfg(target_os = "android")]
    {
        clipboard_android::copy_file_and_image_to_clipboard(
            &path_str,
            Some((&raw_bytes, &final_format)),
            is_animated_gif,
        )?;
        Ok(true)
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "android")))]
    {
        Ok(true)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_emotes,
            import_emotes,
            import_folder,
            get_storage_path,
            set_storage_path,
            delete_emote,
            update_tag,
            resize_emote,
            convert_emote,
            copy_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
