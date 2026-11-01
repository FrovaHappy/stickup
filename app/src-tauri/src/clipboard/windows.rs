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

