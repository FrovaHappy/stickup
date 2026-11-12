#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "android")]
mod android;

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{AnimationDecoder, Frame, ImageFormat};
use std::fs;
use std::path::{Path, PathBuf};

pub enum ClipboardPayload {
    LocalPaths(Vec<PathBuf>),
    RawImageBytes(Vec<u8>),
    #[cfg(windows)]
    RgbaImage {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    },
    TextUrl(String),
}

pub fn read_clipboard_payload() -> Result<ClipboardPayload, String> {
    #[cfg(windows)]
    {
        windows::read_from_clipboard()
    }
    #[cfg(target_os = "linux")]
    {
        linux::read_from_clipboard()
    }
    #[cfg(target_os = "android")]
    {
        android::read_from_clipboard()
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "android")))]
    {
        Err("Lectura de portapapeles no soportada en esta plataforma".to_string())
    }
}

pub fn fit_dimensions(orig_w: u32, orig_h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if orig_w == 0 || orig_h == 0 {
        return (max_w.max(1), max_h.max(1));
    }
    if max_w == 0 || max_h == 0 {
        return (orig_w.max(1), orig_h.max(1));
    }
    let ratio = (max_w as f64 / orig_w as f64).min(max_h as f64 / orig_h as f64);
    let new_w = ((orig_w as f64 * ratio).round() as u32).max(1);
    let new_h = ((orig_h as f64 * ratio).round() as u32).max(1);
    (new_w, new_h)
}

pub fn copy_emote(
    path: &str,
    format: &str,
    target_width: u32,
    target_height: u32,
) -> Result<bool, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err("El archivo del emote no existe".to_string());
    }

    let orig_ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let req_format = format.to_lowercase();
    let needs_resize = target_width > 0 && target_height > 0;
    let is_animated_gif = orig_ext == "gif" && req_format == "gif";
    let orig_dims = image::image_dimensions(p).ok();
    let (target_w, target_h) = if needs_resize {
        if let Some((ow, oh)) = orig_dims {
            fit_dimensions(ow, oh, target_width, target_height)
        } else {
            (target_width, target_height)
        }
    } else {
        (target_width, target_height)
    };

    let is_same_dimensions = if let Some((w, h)) = orig_dims {
        w == target_w && h == target_h
    } else {
        false
    };

    let final_path: PathBuf;
    let raw_bytes: Vec<u8>;
    let final_format: String;
    #[cfg(windows)]
    let mut rgba_data: Option<(Vec<u8>, u32, u32)> = None;

    if is_animated_gif && (!needs_resize || is_same_dimensions) {
        final_path = p.to_path_buf();
        raw_bytes = fs::read(p).map_err(|e| e.to_string())?;
        final_format = "gif".to_string();
    } else if is_animated_gif && needs_resize {
        let file = fs::File::open(p).map_err(|e| e.to_string())?;
        let reader = std::io::BufReader::new(file);
        let decoder = GifDecoder::new(reader).map_err(|e| e.to_string())?;
        let frames = decoder
            .into_frames()
            .collect_frames()
            .map_err(|e| e.to_string())?;

        let mut resized_frames = Vec::new();
        for frame in frames {
            let delay = frame.delay();
            let img = image::DynamicImage::ImageRgba8(frame.into_buffer());
            let (frame_target_w, frame_target_h) =
                fit_dimensions(img.width(), img.height(), target_width, target_height);
            let resized = image::imageops::resize(
                &img,
                frame_target_w,
                frame_target_h,
                FilterType::Lanczos3,
            );
            resized_frames.push(Frame::from_parts(resized, 0, 0, delay));
        }

        let temp_dir = std::env::temp_dir().join("stickup_clipboard");
        let _ = fs::create_dir_all(&temp_dir);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("emote");
        let temp_file = temp_dir.join(format!("{}_{}x{}.gif", stem, target_w, target_h));

        let out_file = fs::File::create(&temp_file).map_err(|e| e.to_string())?;
        let mut encoder = GifEncoder::new(out_file);
        let repeat_mode = if let Ok(bytes) = fs::read(p) {
            if let Ok(status) = crate::services::gif::inspect_gif_loop(&bytes) {
                if status.is_infinite {
                    Repeat::Infinite
                } else {
                    Repeat::Finite(status.loop_count.unwrap_or(1))
                }
            } else {
                Repeat::Infinite
            }
        } else {
            Repeat::Infinite
        };
        encoder
            .set_repeat(repeat_mode)
            .map_err(|e| e.to_string())?;
        encoder
            .encode_frames(resized_frames.into_iter())
            .map_err(|e| e.to_string())?;

        raw_bytes = fs::read(&temp_file).map_err(|e| e.to_string())?;
        final_path = temp_file;
        final_format = "gif".to_string();
    } else {
        let img = image::open(p).map_err(|e| format!("Error cargando imagen: {}", e))?;
        let (fitted_w, fitted_h) = if needs_resize {
            fit_dimensions(img.width(), img.height(), target_width, target_height)
        } else {
            (img.width(), img.height())
        };
        let processed = if needs_resize
            && (img.width() != fitted_w || img.height() != fitted_h)
        {
            image::imageops::resize(&img, fitted_w, fitted_h, FilterType::Lanczos3)
        } else {
            img.to_rgba8()
        };

        let (w, h) = (processed.width(), processed.height());
        #[cfg(windows)]
        {
            rgba_data = Some((processed.clone().into_raw(), w, h));
        }

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

        if orig_ext == req_format && !needs_resize {
            final_path = p.to_path_buf();
        } else {
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
        windows::copy_file_and_image_to_clipboard(
            &path_str,
            Some((&raw_bytes, &final_format)),
            rgba_ref,
            is_animated_gif,
        )?;
        Ok(true)
    }

    #[cfg(target_os = "linux")]
    {
        linux::copy_file_and_image_to_clipboard(
            &path_str,
            Some((&raw_bytes, &final_format)),
            is_animated_gif,
        )?;
        Ok(true)
    }

    #[cfg(target_os = "android")]
    {
        android::copy_file_and_image_to_clipboard(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_dimensions_landscape() {
        // 300x200 inside 128x128 bounding box => aspect ratio 1.5, fits to 128x85
        let (w, h) = fit_dimensions(300, 200, 128, 128);
        assert_eq!(w, 128);
        assert_eq!(h, 85);
    }

    #[test]
    fn test_fit_dimensions_portrait() {
        // 200x300 inside 128x128 bounding box => fits to 85x128
        let (w, h) = fit_dimensions(200, 300, 128, 128);
        assert_eq!(w, 85);
        assert_eq!(h, 128);
    }

    #[test]
    fn test_fit_dimensions_square() {
        let (w, h) = fit_dimensions(256, 256, 128, 128);
        assert_eq!(w, 128);
        assert_eq!(h, 128);
    }

    #[test]
    fn test_fit_dimensions_custom_rectangle() {
        // 300x200 inside 200x100
        let (w, h) = fit_dimensions(300, 200, 200, 100);
        assert_eq!(w, 150);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_fit_dimensions_zero_handling() {
        assert_eq!(fit_dimensions(0, 0, 128, 128), (128, 128));
        assert_eq!(fit_dimensions(300, 200, 0, 0), (300, 200));
    }
}

