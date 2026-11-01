use super::ClipboardPayload;

pub fn copy_file_and_image_to_clipboard(
    _file_path: &str,
    _raw_bytes: Option<(&[u8], &str)>,
    _is_animated_gif: bool,
) -> Result<(), String> {
    Ok(())
}

pub fn read_from_clipboard() -> Result<ClipboardPayload, String> {
    Err("Lectura de portapapeles no soportada en Android".to_string())
}

