use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
use ndarray::Array4;
use ort::environment::Environment;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::TensorRef;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

static ORT_ENV: OnceLock<Arc<Environment>> = OnceLock::new();

pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 60; // 1 minuto por defecto
static IDLE_TIMEOUT_SECS: AtomicU64 = AtomicU64::new(DEFAULT_IDLE_TIMEOUT_SECS);
static REAPER_RUNNING: AtomicBool = AtomicBool::new(false);

struct CachedSession {
    path: PathBuf,
    session: Session,
    last_used: Instant,
}

static SESSION_CACHE: Mutex<Option<CachedSession>> = Mutex::new(None);

pub fn get_idle_timeout_secs() -> u64 {
    IDLE_TIMEOUT_SECS.load(Ordering::Relaxed)
}

pub fn set_idle_timeout_secs(secs: u64) {
    IDLE_TIMEOUT_SECS.store(secs, Ordering::Relaxed);
    if secs == 0 {
        clear_session_cache();
    }
}

fn get_or_init_env() -> Result<Arc<Environment>, String> {
    if let Some(env) = ORT_ENV.get() {
        return Ok(Arc::clone(env));
    }
    let env = Arc::new(
        ort::init()
            .with_name("stickup")
            .build()
            .map_err(|e| format!("Error al inicializar runtime de ONNX: {}", e))?,
    );
    let _ = ORT_ENV.set(Arc::clone(&env));
    Ok(env)
}

fn spawn_reaper_if_needed() {
    let timeout_secs = IDLE_TIMEOUT_SECS.load(Ordering::Relaxed);
    if timeout_secs == 0 {
        return;
    }
    if REAPER_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        std::thread::Builder::new()
            .name("stickup-bg-model-reaper".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(10));
                    let cur_timeout = IDLE_TIMEOUT_SECS.load(Ordering::Relaxed);
                    if cur_timeout == 0 {
                        clear_session_cache();
                        REAPER_RUNNING.store(false, Ordering::SeqCst);
                        break;
                    }

                    let mut session_guard = match SESSION_CACHE.lock() {
                        Ok(g) => g,
                        Err(_) => break,
                    };

                    if let Some(ref cached) = *session_guard {
                        if cached.last_used.elapsed() >= Duration::from_secs(cur_timeout) {
                            *session_guard = None;
                            REAPER_RUNNING.store(false, Ordering::SeqCst);
                            break;
                        }
                    } else {
                        REAPER_RUNNING.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            })
            .ok();
    }
}

pub fn clear_session_cache() {
    if let Ok(mut lock) = SESSION_CACHE.lock() {
        *lock = None;
    }
}

pub fn is_session_cached() -> bool {
    if let Ok(guard) = SESSION_CACHE.lock() {
        guard.is_some()
    } else {
        false
    }
}

pub fn check_and_prune_idle_session(max_idle: Duration) -> bool {
    let mut guard = match SESSION_CACHE.lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    if let Some(cached) = &*guard {
        if cached.last_used.elapsed() >= max_idle {
            *guard = None;
            return true;
        }
    }
    false
}

pub fn apply_temperature(p: f32, temp: f32) -> f32 {
    let t = temp.clamp(0.05, 3.0);
    if (t - 1.0).abs() < 1e-4 {
        return p;
    }
    let p_clamped = p.clamp(1e-5, 1.0 - 1e-5);
    let logit = (p_clamped / (1.0 - p_clamped)).ln();
    let scaled = logit / t;
    (1.0 / (1.0 + (-scaled).exp())).clamp(0.0, 1.0)
}

pub fn remove_background(
    img: &DynamicImage,
    model_path: &Path,
    model_id: &str,
    temperature: Option<f32>,
) -> Result<RgbaImage, String> {
    if !model_path.exists() {
        return Err(format!(
            "El archivo del modelo no existe en: {}",
            model_path.to_string_lossy()
        ));
    }

    let (orig_w, orig_h) = img.dimensions();
    if orig_w == 0 || orig_h == 0 {
        return Err("La imagen tiene dimensiones inválidas (0x0).".to_string());
    }

    let is_birefnet = model_id.contains("birefnet")
        || model_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.contains("birefnet"))
            .unwrap_or(false);

    // 1. Redimensionar a 1024x1024
    let resized_rgb = img
        .resize_exact(1024, 1024, FilterType::Triangle)
        .to_rgb8();

    // 2. Preprocesar según arquitectura del modelo
    let mut input_tensor = Array4::<f32>::zeros((1, 3, 1024, 1024));

    if is_birefnet {
        // BiRefNet usa normalización ImageNet
        let mean = [0.485, 0.456, 0.406];
        let std = [0.229, 0.224, 0.225];
        for y in 0..1024 {
            for x in 0..1024 {
                let px = resized_rgb.get_pixel(x as u32, y as u32);
                input_tensor[[0, 0, y, x]] = ((px[0] as f32 / 255.0) - mean[0]) / std[0];
                input_tensor[[0, 1, y, x]] = ((px[1] as f32 / 255.0) - mean[1]) / std[1];
                input_tensor[[0, 2, y, x]] = ((px[2] as f32 / 255.0) - mean[2]) / std[2];
            }
        }
    } else {
        // RMBG usa normalización media 0.5, std 1.0
        for y in 0..1024 {
            for x in 0..1024 {
                let px = resized_rgb.get_pixel(x as u32, y as u32);
                input_tensor[[0, 0, y, x]] = (px[0] as f32 / 255.0) - 0.5;
                input_tensor[[0, 1, y, x]] = (px[1] as f32 / 255.0) - 0.5;
                input_tensor[[0, 2, y, x]] = (px[2] as f32 / 255.0) - 0.5;
            }
        }
    }

    // 3. Obtener o crear sesión ONNX Runtime (reutilizando si no cambió la ruta)
    let mut session_guard = SESSION_CACHE.lock().map_err(|e| e.to_string())?;
    let needs_reload = match &*session_guard {
        Some(cached) => cached.path != model_path,
        None => true,
    };

    if needs_reload {
        let env = get_or_init_env()?;
        let session = Session::builder(&env)
            .map_err(|e| format!("Error al crear builder de ONNX Session: {}", e))?
            .with_optimization_level(GraphOptimizationLevel::Level1)
            .map_err(|e| format!("Error de optimización en ONNX: {}", e))?
            .with_memory_pattern(false)
            .map_err(|e| format!("Error en memory pattern: {}", e))?
            .with_config_entry("session.arena_extend_strategy", "kSameAsRequested")
            .map_err(|e| format!("Error en arena extend: {}", e))?
            .with_intra_threads(2)
            .map_err(|e| format!("Error al configurar hilos en ONNX: {}", e))?
            .commit_from_file(model_path)
            .map_err(|e| format!("Error al cargar modelo ONNX desde archivo: {}", e))?;

        *session_guard = Some(CachedSession {
            path: model_path.to_path_buf(),
            session,
            last_used: Instant::now(),
        });
    } else if let Some(ref mut cached) = *session_guard {
        cached.last_used = Instant::now();
    }

    spawn_reaper_if_needed();

    let session = &mut session_guard.as_mut().unwrap().session;

    // 4. Ejecutar inferencia
    let input_view = TensorRef::from_array_view(&input_tensor)
        .map_err(|e| format!("Error al crear vista de tensor para ONNX: {}", e))?;

    let outputs = session
        .run(ort::inputs![input_view])
        .map_err(|e| format!("Error durante la inferencia ONNX: {}", e))?;

    // 5. Extraer máscara de salida [1, 1, 1024, 1024]
    let (_shape, data) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Error al extraer tensor de salida: {}", e))?;

    if data.len() < 1024 * 1024 {
        return Err("El tensor de salida tiene un tamaño inesperado.".to_string());
    }

    // Copiar el buffer de máscara de salida a un vector propio para poder soltar los tensores ONNX y la sesión inmediatamente
    let mask_data: Vec<f32> = data.to_vec();
    drop(outputs);
    drop(session_guard);

    // Si el timeout configurado es instantáneo (0 segundos), descargamos el modelo de RAM de inmediato
    if IDLE_TIMEOUT_SECS.load(Ordering::Relaxed) == 0 {
        clear_session_cache();
    }

    let temp = temperature.unwrap_or(1.0);
    let need_temp = (temp - 1.0).abs() > 1e-4;

    let mut mask_1024: GrayImage = ImageBuffer::new(1024, 1024);

    if is_birefnet {
        // En BiRefNet la salida son logits: aplicamos sigmoide y luego temperatura
        for y in 0..1024 {
            for x in 0..1024 {
                let logit = mask_data[y * 1024 + x];
                let prob = (1.0 / (1.0 + (-logit).exp())).clamp(0.0, 1.0);
                let final_val = if need_temp {
                    apply_temperature(prob, temp)
                } else {
                    prob
                };
                let alpha = (final_val * 255.0) as u8;
                mask_1024.put_pixel(x as u32, y as u32, Luma([alpha]));
            }
        }
    } else {
        // En RMBG normalizamos min-max y luego aplicamos temperatura
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;
        for &v in mask_data.iter().take(1024 * 1024) {
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }
        let range = if max_val > min_val {
            max_val - min_val
        } else {
            1.0
        };

        for y in 0..1024 {
            for x in 0..1024 {
                let val = mask_data[y * 1024 + x];
                let normalized = ((val - min_val) / range).clamp(0.0, 1.0);
                let final_val = if need_temp {
                    apply_temperature(normalized, temp)
                } else {
                    normalized
                };
                let alpha = (final_val * 255.0) as u8;
                mask_1024.put_pixel(x as u32, y as u32, Luma([alpha]));
            }
        }
    }

    // 6. Redimensionar máscara al tamaño original de la imagen
    let full_mask = image::imageops::resize(
        &mask_1024,
        orig_w,
        orig_h,
        FilterType::Triangle,
    );

    // 7. Aplicar canal alfa a la imagen original
    let mut output: RgbaImage = ImageBuffer::new(orig_w, orig_h);
    let original_rgba = img.to_rgba8();

    for y in 0..orig_h {
        for x in 0..orig_w {
            let src_px = original_rgba.get_pixel(x, y);
            let mask_alpha = full_mask.get_pixel(x, y)[0] as f32 / 255.0;
            let src_alpha = src_px[3] as f32 / 255.0;
            let combined_alpha = (mask_alpha * src_alpha * 255.0).clamp(0.0, 255.0) as u8;

            output.put_pixel(
                x,
                y,
                Rgba([src_px[0], src_px[1], src_px[2], combined_alpha]),
            );
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_apply_temperature_neutral() {
        let p = 0.65;
        let res = apply_temperature(p, 1.0);
        assert!((res - p).abs() < 1e-3);
    }

    #[test]
    fn test_apply_temperature_sharp() {
        let p_high = 0.7;
        let res_high = apply_temperature(p_high, 0.5);
        assert!(res_high > p_high);

        let p_low = 0.3;
        let res_low = apply_temperature(p_low, 0.5);
        assert!(res_low < p_low);
    }

    #[test]
    fn test_apply_temperature_soft() {
        let p_high = 0.7;
        let res_high = apply_temperature(p_high, 2.0);
        assert!(res_high < p_high);

        let p_low = 0.3;
        let res_low = apply_temperature(p_low, 2.0);
        assert!(res_low > p_low);
    }

    #[test]
    fn test_remove_background_missing_model() {
        let fake_path = Path::new("non_existent_model_file.onnx");
        let img = DynamicImage::ImageRgb8(RgbImage::new(100, 100));
        let res = remove_background(&img, fake_path, "birefnet_lite", None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("no existe"));
    }

    #[test]
    fn test_remove_background_zero_dim() {
        let fake_path = Path::new("fake.onnx");
        let img = DynamicImage::ImageRgb8(RgbImage::new(0, 0));
        let res = remove_background(&img, fake_path, "birefnet_lite", Some(1.0));
        assert!(res.is_err());
    }

    #[test]
    fn test_session_cache_prune_and_clear() {
        // Al inicio o tras clear no hay sesión
        clear_session_cache();
        assert!(!is_session_cached());
        assert!(!check_and_prune_idle_session(Duration::from_secs(0)));
    }

    #[test]
    fn test_set_and_get_idle_timeout_secs() {
        set_idle_timeout_secs(60);
        assert_eq!(get_idle_timeout_secs(), 60);

        set_idle_timeout_secs(0);
        assert_eq!(get_idle_timeout_secs(), 0);
        assert!(!is_session_cached());

        set_idle_timeout_secs(DEFAULT_IDLE_TIMEOUT_SECS);
        assert_eq!(get_idle_timeout_secs(), DEFAULT_IDLE_TIMEOUT_SECS);
    }
}
