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

    let final_path: PathBuf;
    let raw_bytes: Vec<u8>;
    let final_format: String;
    let mut rgba_data: Option<(Vec<u8>, u32, u32)> = None;

    if is_animated_gif && !needs_resize {
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
            let resized =
                image::imageops::resize(&img, target_width, target_height, FilterType::Lanczos3);
            resized_frames.push(Frame::from_parts(resized, 0, 0, delay));
        }

        let temp_dir = std::env::temp_dir().join("stickup_clipboard");
        let _ = fs::create_dir_all(&temp_dir);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("emote");
        let temp_file = temp_dir.join(format!("{}_{}x{}.gif", stem, target_width, target_height));

        let out_file = fs::File::create(&temp_file).map_err(|e| e.to_string())?;
        let mut encoder = GifEncoder::new(out_file);
        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(|e| e.to_string())?;
        encoder
            .encode_frames(resized_frames.into_iter())
            .map_err(|e| e.to_string())?;

        raw_bytes = fs::read(&temp_file).map_err(|e| e.to_string())?;
        final_path = temp_file;
        final_format = "gif".to_string();
    } else {
        let img = image::open(p).map_err(|e| format!("Error cargando imagen: {}", e))?;
        let processed = if needs_resize
            && (img.width() != target_width || img.height() != target_height)
        {
            image::imageops::resize(&img, target_width, target_height, FilterType::Lanczos3)
        } else {
            img.to_rgba8()
        };

        let (w, h) = (processed.width(), processed.height());
        rgba_data = Some((processed.clone().into_raw(), w, h));

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

