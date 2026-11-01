use super::ClipboardPayload;
use std::fs;
use std::io::Write;
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
    Err("El portapapeles no contiene una imagen compatible".to_string())
}

