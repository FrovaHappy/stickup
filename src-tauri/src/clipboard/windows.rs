use super::ClipboardPayload;
use std::ffi::CString;
use std::mem::size_of;
use std::path::PathBuf;
use std::ptr;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatA, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GHND, GMEM_MOVEABLE,
};
use windows_sys::Win32::UI::Shell::{DragQueryFileW, DROPFILES, HDROP};

const CF_DIB: u32 = 8;
const CF_UNICODETEXT: u32 = 13;
const CF_HDROP: u32 = 15;

pub fn copy_file_and_image_to_clipboard(
    file_path: &str,
    raw_bytes: Option<(&[u8], &str)>,
    img_rgba: Option<(&[u8], u32, u32)>,
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

        // 2. Formato MIME registrado (e.g. "image/gif", "GIF", "image/webp", "PNG")
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

        // 3. CF_DIB: Solo para imágenes estáticas.
        // Si es un GIF animado, se omite para evitar que Discord/Telegram congelen la animación.
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

pub fn read_from_clipboard() -> Result<ClipboardPayload, String> {
    unsafe {
        if OpenClipboard(0 as HWND) == 0 {
            return Err("No se pudo abrir el portapapeles de Windows".to_string());
        }

        // 1. CF_HDROP: Archivo(s) copiado(s) desde el Explorador, Discord, Telegram, etc.
        if IsClipboardFormatAvailable(CF_HDROP) != 0 {
            let h_data = GetClipboardData(CF_HDROP);
            if !h_data.is_null() {
                let h_drop = h_data as HDROP;
                let len = DragQueryFileW(h_drop, 0, ptr::null_mut(), 0);
                if len > 0 {
                    let mut buf = vec![0u16; (len + 1) as usize];
                    let copied = DragQueryFileW(h_drop, 0, buf.as_mut_ptr(), len + 1);
                    if copied > 0 {
                        buf.truncate(copied as usize);
                        let path_str = String::from_utf16_lossy(&buf);
                        let p = PathBuf::from(&path_str);
                        if p.exists() && p.is_file() {
                            CloseClipboard();
                            return Ok(ClipboardPayload::FilePath(p));
                        }
                    }
                }
            }
        }

        // 2. Formatos MIME registrados (conservan animación GIF, transparencia PNG/WebP, etc.)
        let mime_candidates = [
            "image/gif",
            "GIF",
            "image/webp",
            "WEBP",
            "PNG",
            "image/png",
            "image/jpeg",
            "image/jpg",
            "JFIF",
        ];

        for fmt_name in mime_candidates {
            if let Ok(c_str) = CString::new(fmt_name) {
                let cf = RegisterClipboardFormatA(c_str.as_ptr() as *const u8);
                if cf != 0 && IsClipboardFormatAvailable(cf) != 0 {
                    let h_data = GetClipboardData(cf);
                    if !h_data.is_null() {
                        let size = GlobalSize(h_data);
                        let ptr_data = GlobalLock(h_data) as *const u8;
                        if !ptr_data.is_null() && size > 0 {
                            let mut bytes = vec![0u8; size];
                            ptr::copy_nonoverlapping(ptr_data, bytes.as_mut_ptr(), size);
                            GlobalUnlock(h_data);
                            CloseClipboard();
                            return Ok(ClipboardPayload::RawImageBytes(bytes));
                        }
                        if !ptr_data.is_null() {
                            GlobalUnlock(h_data);
                        }
                    }
                }
            }
        }

        // 3. CF_DIB: Capturas de pantalla, "Copiar imagen" desde navegador o visor
        if IsClipboardFormatAvailable(CF_DIB) != 0 {
            let h_dib = GetClipboardData(CF_DIB);
            if !h_dib.is_null() {
                let total_size = GlobalSize(h_dib);
                let p_dib = GlobalLock(h_dib) as *const u8;
                if !p_dib.is_null() && total_size >= size_of::<BITMAPINFOHEADER>() {
                    let bmi = &*(p_dib as *const BITMAPINFOHEADER);
                    let width = bmi.biWidth.unsigned_abs();
                    let height = bmi.biHeight.unsigned_abs();
                    let is_bottom_up = bmi.biHeight > 0;
                    let bit_count = bmi.biBitCount;
                    let header_size = bmi.biSize as usize;

                    if width > 0 && height > 0 && (bit_count == 32 || bit_count == 24) {
                        let bytes_per_px = (bit_count / 8) as u32;
                        let row_stride = ((width * (bit_count as u32) + 31) / 32) * 4;
                        let mut pixel_offset = header_size;
                        // Si biCompression == BI_BITFIELDS (3), hay 12 bytes extra de máscaras de color
                        if bmi.biCompression == 3 && total_size >= header_size + 12 {
                            pixel_offset += 12;
                        }

                        let needed = pixel_offset + (row_stride * height) as usize;
                        if total_size >= needed {
                            let pixels = p_dib.add(pixel_offset);
                            let mut rgba = vec![0u8; (width * height * 4) as usize];
                            let mut any_nonzero_alpha = false;

                            for y in 0..height {
                                let src_y = if is_bottom_up { height - 1 - y } else { y };
                                let src_row = (src_y * row_stride) as usize;
                                let dst_row = (y * width * 4) as usize;

                                for x in 0..width {
                                    let s = src_row + (x * bytes_per_px) as usize;
                                    let d = dst_row + (x * 4) as usize;
                                    let b = *pixels.add(s);
                                    let g = *pixels.add(s + 1);
                                    let r = *pixels.add(s + 2);
                                    let a = if bit_count == 32 {
                                        let alpha = *pixels.add(s + 3);
                                        if alpha != 0 {
                                            any_nonzero_alpha = true;
                                        }
                                        alpha
                                    } else {
                                        255
                                    };

                                    rgba[d] = r;
                                    rgba[d + 1] = g;
                                    rgba[d + 2] = b;
                                    rgba[d + 3] = a;
                                }
                            }

                            if bit_count == 32 && !any_nonzero_alpha {
                                for px in rgba.chunks_exact_mut(4) {
                                    px[3] = 255;
                                }
                            }

                            GlobalUnlock(h_dib);
                            CloseClipboard();
                            return Ok(ClipboardPayload::RgbaImage {
                                rgba,
                                width,
                                height,
                            });
                        }
                    }
                    GlobalUnlock(h_dib);
                }
            }
        }

        // 4. CF_UNICODETEXT: URL copiada en el portapapeles
        if IsClipboardFormatAvailable(CF_UNICODETEXT) != 0 {
            let h_text = GetClipboardData(CF_UNICODETEXT);
            if !h_text.is_null() {
                let byte_size = GlobalSize(h_text);
                let p_text = GlobalLock(h_text) as *const u16;
                if !p_text.is_null() && byte_size >= 2 {
                    let max_u16 = byte_size / 2;
                    let slice = std::slice::from_raw_parts(p_text, max_u16);
                    let end = slice.iter().position(|&c| c == 0).unwrap_or(max_u16);
                    let text = String::from_utf16_lossy(&slice[..end]).trim().to_string();
                    GlobalUnlock(h_text);
                    if text.starts_with("http://") || text.starts_with("https://") {
                        CloseClipboard();
                        return Ok(ClipboardPayload::TextUrl(text));
                    }
                } else if !p_text.is_null() {
                    GlobalUnlock(h_text);
                }
            }
        }

        CloseClipboard();
        Err("El portapapeles no contiene una imagen ni una URL válida".to_string())
    }
}

