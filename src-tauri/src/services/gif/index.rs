use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GifLoopStatus {
    pub is_infinite: bool,
    pub loop_count: Option<u16>,
    pub has_netscape_ext: bool,
}

pub const NETSCAPE_LOOP_BLOCK_INFINITE: [u8; 19] = [
    0x21, 0xFF, 0x0B, b'N', b'E', b'T', b'S', b'C', b'A', b'P', b'E', b'2', b'.', b'0', 0x03,
    0x01, 0x00, 0x00, 0x00,
];

pub const NETSCAPE_LOOP_BLOCK_ONCE: [u8; 19] = [
    0x21, 0xFF, 0x0B, b'N', b'E', b'T', b'S', b'C', b'A', b'P', b'E', b'2', b'.', b'0', 0x03,
    0x01, 0x01, 0x00, 0x00,
];

struct NetscapeLoopLocation {
    pub loop_count_offset: usize,
    pub loop_count: u16,
}

fn find_netscape_loop(bytes: &[u8]) -> (Option<NetscapeLoopLocation>, usize) {
    if bytes.len() < 13 || (&bytes[0..6] != b"GIF89a" && &bytes[0..6] != b"GIF87a") {
        return (None, 0);
    }

    let packed = bytes[10];
    let gct_flag = (packed & 0x80) != 0;
    let gct_size = if gct_flag {
        let n = (packed & 0x07) as usize;
        3 * (1 << (n + 1))
    } else {
        0
    };

    let insert_pos = 13 + gct_size;
    if insert_pos > bytes.len() {
        return (None, 0);
    }

    let mut pos = insert_pos;
    while pos < bytes.len() {
        match bytes[pos] {
            0x3B => {
                // Trailer
                break;
            }
            0x21 => {
                // Extension Introducer
                pos += 1;
                if pos >= bytes.len() {
                    break;
                }
                let ext_label = bytes[pos];
                pos += 1;

                if ext_label == 0xFF {
                    // Application Extension
                    if pos < bytes.len() {
                        let block_len = bytes[pos] as usize;
                        pos += 1;
                        if pos + block_len <= bytes.len() {
                            let app_ident = &bytes[pos..pos + block_len];
                            pos += block_len;

                            let is_loop_ext = block_len == 11
                                && (&app_ident[0..11] == b"NETSCAPE2.0"
                                    || &app_ident[0..11] == b"ANIMEXTS1.0");

                            while pos < bytes.len() {
                                let sub_len = bytes[pos] as usize;
                                pos += 1;
                                if sub_len == 0 {
                                    break;
                                }
                                if pos + sub_len > bytes.len() {
                                    break;
                                }

                                if is_loop_ext && sub_len == 3 && bytes[pos] == 1 {
                                    let loop_count =
                                        u16::from_le_bytes([bytes[pos + 1], bytes[pos + 2]]);
                                    return (
                                        Some(NetscapeLoopLocation {
                                            loop_count_offset: pos + 1,
                                            loop_count,
                                        }),
                                        insert_pos,
                                    );
                                }
                                pos += sub_len;
                            }
                            continue;
                        }
                    }
                }

                // Skip other extension sub-blocks
                while pos < bytes.len() {
                    let sub_len = bytes[pos] as usize;
                    pos += 1;
                    if sub_len == 0 {
                        break;
                    }
                    pos += sub_len;
                }
            }
            0x2C => {
                // Image Descriptor: 10 bytes (including 0x2C)
                if pos + 10 > bytes.len() {
                    break;
                }
                let img_packed = bytes[pos + 9];
                pos += 10;

                // Local color table
                if (img_packed & 0x80) != 0 {
                    let n = (img_packed & 0x07) as usize;
                    pos += 3 * (1 << (n + 1));
                    if pos > bytes.len() {
                        break;
                    }
                }

                // LZW code size
                if pos >= bytes.len() {
                    break;
                }
                pos += 1;

                // Image data sub-blocks
                while pos < bytes.len() {
                    let sub_len = bytes[pos] as usize;
                    pos += 1;
                    if sub_len == 0 {
                        break;
                    }
                    pos += sub_len;
                }
            }
            _ => {
                pos += 1;
            }
        }
    }

    (None, insert_pos)
}

pub fn inspect_gif_loop(bytes: &[u8]) -> Result<GifLoopStatus, String> {
    if bytes.len() < 13 || (&bytes[0..6] != b"GIF89a" && &bytes[0..6] != b"GIF87a") {
        return Err("No es un archivo GIF válido".to_string());
    }

    let (found, _) = find_netscape_loop(bytes);
    if let Some(loc) = found {
        Ok(GifLoopStatus {
            is_infinite: loc.loop_count == 0,
            loop_count: Some(loc.loop_count),
            has_netscape_ext: true,
        })
    } else {
        // En ausencia de extensión Netscape, el GIF estándar se reproduce 1 vez y se detiene.
        Ok(GifLoopStatus {
            is_infinite: false,
            loop_count: None,
            has_netscape_ext: false,
        })
    }
}

pub fn patch_gif_loop(bytes: &[u8], infinite: bool) -> Result<Vec<u8>, String> {
    if bytes.len() < 13 || (&bytes[0..6] != b"GIF89a" && &bytes[0..6] != b"GIF87a") {
        return Err("No es un archivo GIF válido".to_string());
    }

    let (found, insert_pos) = find_netscape_loop(bytes);
    if let Some(loc) = found {
        // Modificación directa in-place de los 2 bytes de repetición
        let mut modified = bytes.to_vec();
        let val: u16 = if infinite { 0 } else { 1 };
        let count_bytes = val.to_le_bytes();
        modified[loc.loop_count_offset] = count_bytes[0];
        modified[loc.loop_count_offset + 1] = count_bytes[1];
        Ok(modified)
    } else {
        // No tiene extensión Netscape -> se inserta de forma limpia y estándar
        if insert_pos == 0 || insert_pos > bytes.len() {
            return Err("Estructura GIF inválida para inserción de bucle".to_string());
        }

        let mut out = Vec::with_capacity(bytes.len() + 19);
        out.extend_from_slice(&bytes[..insert_pos]);
        if infinite {
            out.extend_from_slice(&NETSCAPE_LOOP_BLOCK_INFINITE);
        } else {
            out.extend_from_slice(&NETSCAPE_LOOP_BLOCK_ONCE);
        }
        out.extend_from_slice(&bytes[insert_pos..]);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_gif_without_loop() -> Vec<u8> {
        let mut gif = Vec::new();
        // Header
        gif.extend_from_slice(b"GIF89a");
        // LSD: 10x10, GCT flag=1, GCT size=2 colors (packed=0x80)
        gif.extend_from_slice(&[10, 0, 10, 0, 0x80, 0, 0]);
        // GCT: 2 colors (6 bytes)
        gif.extend_from_slice(&[255, 0, 0, 0, 0, 255]);
        // Graphic Control Extension
        gif.extend_from_slice(&[0x21, 0xF9, 0x04, 0x00, 10, 0, 0x00, 0x00]);
        // Image Descriptor: 10x10
        gif.extend_from_slice(&[0x2C, 0, 0, 0, 0, 10, 0, 10, 0, 0x00]);
        // LZW min code size
        gif.push(0x02);
        // Image data sub-block
        gif.extend_from_slice(&[0x02, 0x44, 0x01, 0x00]);
        // Trailer
        gif.push(0x3B);
        gif
    }

    #[test]
    fn test_inspect_gif_without_loop() {
        let gif = create_dummy_gif_without_loop();
        let status = inspect_gif_loop(&gif).expect("Debe inspeccionar correctamente");
        assert!(!status.is_infinite);
        assert!(!status.has_netscape_ext);
        assert_eq!(status.loop_count, None);
    }

    #[test]
    fn test_patch_to_infinite_inserts_block_when_missing() {
        let gif = create_dummy_gif_without_loop();
        let patched = patch_gif_loop(&gif, true).expect("Debe parchear a infinito");
        let status = inspect_gif_loop(&patched).expect("Debe inspeccionar correctamente");
        assert!(status.is_infinite);
        assert!(status.has_netscape_ext);
        assert_eq!(status.loop_count, Some(0));
    }

    #[test]
    fn test_patch_to_finite_sets_loop_count_to_one() {
        let gif = create_dummy_gif_without_loop();
        // Primero hacemos infinito
        let infinite_gif = patch_gif_loop(&gif, true).unwrap();
        assert!(inspect_gif_loop(&infinite_gif).unwrap().is_infinite);

        // Ahora cambiamos a finito (reproducir 1 vez y frenar)
        let finite_gif = patch_gif_loop(&infinite_gif, false).expect("Debe parchear a finito");
        let status = inspect_gif_loop(&finite_gif).unwrap();
        assert!(!status.is_infinite);
        assert!(status.has_netscape_ext);
        assert_eq!(status.loop_count, Some(1));

        // Y de nuevo a infinito in-place
        let infinite_again = patch_gif_loop(&finite_gif, true).unwrap();
        let status_again = inspect_gif_loop(&infinite_again).unwrap();
        assert!(status_again.is_infinite);
        assert_eq!(status_again.loop_count, Some(0));
    }

    #[test]
    fn test_rejects_invalid_gif_header() {
        let invalid = b"NOT_A_GIF_FILE_AT_ALL";
        assert!(inspect_gif_loop(invalid).is_err());
        assert!(patch_gif_loop(invalid, true).is_err());
    }
}

