use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MIN_VALID_MODEL_SIZE: u64 = 15_000_000; // Al menos 15MB para considerar el archivo íntegro

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelPresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub download_url: String,
    pub approx_size_mb: u32,
    pub is_downloaded: bool,
    pub file_size_bytes: u64,
    pub is_active: bool,
}

pub struct ModelPresetDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub filename: &'static str,
    pub download_url: &'static str,
    pub approx_size_mb: u32,
}

pub const AVAILABLE_PRESETS: &[ModelPresetDef] = &[
    ModelPresetDef {
        id: "birefnet_lite",
        name: "BiRefNet Lite (Máxima Calidad - SOTA)",
        description: "Estado del arte en recorte y segmentación de alta resolución. Máxima fidelidad en cabello, anime, stickers y bordes complejos.",
        filename: "birefnet-lite.onnx",
        download_url: "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/main/onnx/model.onnx",
        approx_size_mb: 224,
    },
    ModelPresetDef {
        id: "rmbg14_full",
        name: "RMBG-1.4 (Completo FP32)",
        description: "Modelo oficial de BRIA AI en precisión completa de 32 bits sin la degradación de cuantización.",
        filename: "rmbg-1.4-full.onnx",
        download_url: "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx",
        approx_size_mb: 176,
    },
    ModelPresetDef {
        id: "rmbg14_quantized",
        name: "RMBG-1.4 (Ligero INT8)",
        description: "Versión comprimida a 8 bits recomendada solo para ahorrar espacio en disco o conexiones muy limitadas.",
        filename: "rmbg-1.4-quantized.onnx",
        download_url: "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model_quantized.onnx",
        approx_size_mb: 43,
    },
];

pub fn find_preset(id: &str) -> Option<&'static ModelPresetDef> {
    AVAILABLE_PRESETS.iter().find(|p| p.id == id)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelStatus {
    pub is_downloaded: bool,
    pub file_size_bytes: u64,
    pub model_path: String,
    pub active_model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f32,
    pub done: bool,
}

pub fn get_preset_path(base_dir: &Path, model_id: &str) -> PathBuf {
    let filename = find_preset(model_id)
        .map(|p| p.filename)
        .unwrap_or("birefnet-lite.onnx");
    base_dir.join(filename)
}

pub fn get_presets_status(base_dir: &Path, active_id: &str) -> Vec<ModelPresetInfo> {
    AVAILABLE_PRESETS
        .iter()
        .map(|preset| {
            let path = base_dir.join(preset.filename);
            let mut is_downloaded = false;
            let mut file_size_bytes = 0;
            if path.exists() {
                if let Ok(meta) = fs::metadata(&path) {
                    if meta.len() >= MIN_VALID_MODEL_SIZE {
                        is_downloaded = true;
                        file_size_bytes = meta.len();
                    }
                }
            }
            ModelPresetInfo {
                id: preset.id.to_string(),
                name: preset.name.to_string(),
                description: preset.description.to_string(),
                filename: preset.filename.to_string(),
                download_url: preset.download_url.to_string(),
                approx_size_mb: preset.approx_size_mb,
                is_downloaded,
                file_size_bytes,
                is_active: preset.id == active_id,
            }
        })
        .collect()
}

pub fn get_model_status_in_dir(base_dir: &Path, active_id: &str) -> ModelStatus {
    let path = get_preset_path(base_dir, active_id);
    if path.exists() {
        if let Ok(meta) = fs::metadata(&path) {
            let size = meta.len();
            if size >= MIN_VALID_MODEL_SIZE {
                return ModelStatus {
                    is_downloaded: true,
                    file_size_bytes: size,
                    model_path: path.to_string_lossy().to_string(),
                    active_model_id: active_id.to_string(),
                };
            }
        }
    }

    ModelStatus {
        is_downloaded: false,
        file_size_bytes: 0,
        model_path: path.to_string_lossy().to_string(),
        active_model_id: active_id.to_string(),
    }
}

pub fn delete_model_preset_in_dir(base_dir: &Path, model_id: &str) -> Result<(), String> {
    let path = get_preset_path(base_dir, model_id);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Error al eliminar el modelo: {}", e))?;
    }
    let filename = find_preset(model_id).map(|p| p.filename).unwrap_or("model.onnx");
    let tmp_path = base_dir.join(format!("{}.tmp", filename));
    if tmp_path.exists() {
        let _ = fs::remove_file(&tmp_path);
    }
    Ok(())
}

pub async fn download_model_with_progress<F>(
    base_dir: &Path,
    model_id: &str,
    mut on_progress: F,
) -> Result<PathBuf, String>
where
    F: FnMut(DownloadProgress),
{
    let preset = find_preset(model_id)
        .ok_or_else(|| format!("Preset de modelo no reconocido: {}", model_id))?;

    if !base_dir.exists() {
        fs::create_dir_all(base_dir)
            .map_err(|e| format!("No se pudo crear el directorio de modelos: {}", e))?;
    }

    let target_path = base_dir.join(preset.filename);
    let tmp_path = base_dir.join(format!("{}.tmp", preset.filename));

    if tmp_path.exists() {
        let _ = fs::remove_file(&tmp_path);
    }

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(900)) // 15 min para conexiones lentas
        .build()
        .map_err(|e| format!("Error al crear cliente HTTP: {}", e))?;

    let response = client.get(preset.download_url).send().await.map_err(|e| {
        if e.is_connect() || e.is_timeout() {
            "No se pudo conectar con el servidor de Hugging Face. Verifica tu conexión a internet.".to_string()
        } else {
            format!("Error de conexión con el servidor de modelos: {}", e)
        }
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "El servidor de Hugging Face devolvió un error ({}: {}). Por favor, intenta de nuevo más tarde.",
            status.as_u16(),
            status.canonical_reason().unwrap_or("No disponible")
        ));
    }

    let total_bytes = response.content_length();
    let mut downloaded_bytes: u64 = 0;

    let mut file = File::create(&tmp_path)
        .map_err(|e| format!("No se pudo crear el archivo temporal de descarga: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!("Error durante la descarga de datos del modelo: {}", e)
        })?;

        file.write_all(&chunk).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!("Error al escribir en disco durante la descarga: {}", e)
        })?;

        downloaded_bytes += chunk.len() as u64;

        let percentage = if let Some(total) = total_bytes {
            if total > 0 {
                ((downloaded_bytes as f64 / total as f64) * 100.0) as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        on_progress(DownloadProgress {
            downloaded_bytes,
            total_bytes,
            percentage,
            done: false,
        });
    }

    file.flush()
        .map_err(|e| format!("Error al sincronizar archivo en disco: {}", e))?;
    drop(file);

    // Validar tamaño mínimo para evitar archivos incompletos
    let meta = fs::metadata(&tmp_path)
        .map_err(|e| format!("No se pudo leer metadatos del archivo descargado: {}", e))?;
    if meta.len() < MIN_VALID_MODEL_SIZE {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "El archivo descargado está incompleto o corrupto (tamaño: {} bytes).",
            meta.len()
        ));
    }

    // Renombrado atómico
    if target_path.exists() {
        let _ = fs::remove_file(&target_path);
    }
    fs::rename(&tmp_path, &target_path)
        .map_err(|e| format!("No se pudo finalizar la instalación del modelo: {}", e))?;

    on_progress(DownloadProgress {
        downloaded_bytes,
        total_bytes: Some(downloaded_bytes),
        percentage: 100.0,
        done: true,
    });

    Ok(target_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_presets_list() {
        assert!(AVAILABLE_PRESETS.len() >= 3);
        assert_eq!(AVAILABLE_PRESETS[0].id, "birefnet_lite");
        assert!(find_preset("birefnet_lite").is_some());
        assert!(find_preset("rmbg14_full").is_some());
        assert!(find_preset("rmbg14_quantized").is_some());
        assert!(find_preset("invalid_preset").is_none());
    }

    #[test]
    fn test_model_status_not_downloaded() {
        let temp_dir = std::env::temp_dir().join(format!("test_bg_model_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let status = get_model_status_in_dir(&temp_dir, "birefnet_lite");
        assert!(!status.is_downloaded);
        assert_eq!(status.file_size_bytes, 0);
        assert_eq!(status.active_model_id, "birefnet_lite");

        let presets = get_presets_status(&temp_dir, "birefnet_lite");
        assert_eq!(presets.len(), AVAILABLE_PRESETS.len());
        assert!(presets[0].is_active);
        assert!(!presets[0].is_downloaded);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_model_status_and_delete() {
        let temp_dir = std::env::temp_dir().join(format!("test_bg_model_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let model_path = get_preset_path(&temp_dir, "birefnet_lite");
        let file = File::create(&model_path).unwrap();
        file.set_len(20_000_000).unwrap();
        drop(file);

        let status = get_model_status_in_dir(&temp_dir, "birefnet_lite");
        assert!(status.is_downloaded);
        assert_eq!(status.file_size_bytes, 20_000_000);

        // Borrar
        let del_res = delete_model_preset_in_dir(&temp_dir, "birefnet_lite");
        assert!(del_res.is_ok());
        assert!(!model_path.exists());

        let status_after = get_model_status_in_dir(&temp_dir, "birefnet_lite");
        assert!(!status_after.is_downloaded);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
