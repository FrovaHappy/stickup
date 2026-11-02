use super::ClipboardPayload;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn copy_file_and_image_to_clipboard(
    file_path: &str,
    raw_bytes: Option<(&[u8], &str)>,
    _is_animated_gif: bool,
) -> Result<(), String> {
    let uri = format!("file://{}\r\n", file_path);

    // 1. Wayland: wl-copy
    if let Ok(mut child) = Command::new("wl-copy")
        .args(["--type", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(uri.as_bytes());
        }
        if let Ok(status) = child.wait() {
            if status.success() {
                return Ok(());
            }
        }
    }

    // 2. X11: xclip con el MIME correspondiente
    let mime_type = match raw_bytes {
        Some((_, "gif")) => "image/gif",
        Some((_, "webp")) => "image/webp",
        _ => "image/png",
    };

    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard", "-target", mime_type])
        .stdin(Stdio::piped())
        .spawn()
    {
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

    // 3. Fallback X11: text/uri-list
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard", "-target", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(uri.as_bytes());
        }
        let _ = child.wait();
    }

    Ok(())
}

pub fn read_from_clipboard() -> Result<ClipboardPayload, String> {
    // 1. Archivo copiado como text/uri-list (Wayland / X11)
    let uri_out = Command::new("wl-paste")
        .args(["--type", "text/uri-list"])
        .output()
        .ok()
        .filter(|o| o.status.success() && !o.stdout.is_empty())
        .or_else(|| {
            Command::new("xclip")
                .args(["-selection", "clipboard", "-t", "text/uri-list", "-o"])
                .output()
                .ok()
                .filter(|o| o.status.success() && !o.stdout.is_empty())
        });

    if let Some(out) = uri_out {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(raw_path) = trimmed.strip_prefix("file://") {
                let p = PathBuf::from(raw_path);
                if p.exists() && p.is_file() {
                    return Ok(ClipboardPayload::FilePath(p));
                }
            }
        }
    }

    // 2. Bytes de imagen (Wayland / X11)
    for mime in ["image/gif", "image/webp", "image/png", "image/jpeg"] {
        if let Ok(out) = Command::new("wl-paste").args(["--type", mime]).output() {
            if out.status.success() && !out.stdout.is_empty() {
                return Ok(ClipboardPayload::RawImageBytes(out.stdout));
            }
        }
        if let Ok(out) = Command::new("xclip")
            .args(["-selection", "clipboard", "-t", mime, "-o"])
            .output()
        {
            if out.status.success() && !out.stdout.is_empty() {
                return Ok(ClipboardPayload::RawImageBytes(out.stdout));
            }
        }
    }

    // 3. Texto plano con URL (Wayland / X11)
    let text_out = Command::new("wl-paste")
        .args(["--no-newline"])
        .output()
        .ok()
        .filter(|o| o.status.success() && !o.stdout.is_empty())
        .or_else(|| {
            Command::new("xclip")
                .args(["-selection", "clipboard", "-o"])
                .output()
                .ok()
                .filter(|o| o.status.success() && !o.stdout.is_empty())
        });

    if let Some(out) = text_out {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if text.starts_with("http://") || text.starts_with("https://") {
            return Ok(ClipboardPayload::TextUrl(text));
        }
    }

    Err("El portapapeles no contiene una imagen compatible".to_string())
}

