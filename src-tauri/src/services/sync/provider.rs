use super::manifest::SyncManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProviderType {
    GoogleDrive,
    OneDrive,
    Mega,
    LocalFolder,
}

impl CloudProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoogleDrive => "googledrive",
            Self::OneDrive => "onedrive",
            Self::Mega => "mega",
            Self::LocalFolder => "localfolder",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "googledrive" | "google_drive" | "gdrive" => Some(Self::GoogleDrive),
            "onedrive" => Some(Self::OneDrive),
            "mega" => Some(Self::Mega),
            "localfolder" | "local_folder" | "folder" => Some(Self::LocalFolder),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync_at: Option<i64>,
    pub provider: Option<String>,
    pub progress: Option<String>,
    pub error: Option<String>,
    pub uploaded_count: u32,
    pub downloaded_count: u32,
}

pub trait CloudStorageProvider: Send + Sync {
    fn provider_type(&self) -> CloudProviderType;
    fn is_configured(&self) -> bool;
    fn get_remote_manifest(&self) -> impl std::future::Future<Output = Result<Option<SyncManifest>, String>> + Send;
    fn save_remote_manifest(&self, manifest: &SyncManifest) -> impl std::future::Future<Output = Result<(), String>> + Send;
    fn upload_file(&self, local_path: &Path, remote_filename: &str) -> impl std::future::Future<Output = Result<(), String>> + Send;
    fn download_file(&self, remote_filename: &str, local_path: &Path) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// Proveedor en memoria para pruebas unitarias deterministas de sincronización
#[derive(Default, Clone)]
pub struct MemoryStorageProvider {
    pub manifest: Arc<Mutex<Option<SyncManifest>>>,
    pub files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl CloudStorageProvider for MemoryStorageProvider {
    fn provider_type(&self) -> CloudProviderType {
        CloudProviderType::Mega
    }

    fn is_configured(&self) -> bool {
        true
    }

    async fn get_remote_manifest(&self) -> Result<Option<SyncManifest>, String> {
        let lock = self.manifest.lock().map_err(|e| e.to_string())?;
        Ok(lock.clone())
    }

    async fn save_remote_manifest(&self, manifest: &SyncManifest) -> Result<(), String> {
        let mut lock = self.manifest.lock().map_err(|e| e.to_string())?;
        *lock = Some(manifest.clone());
        Ok(())
    }

    async fn upload_file(&self, local_path: &Path, remote_filename: &str) -> Result<(), String> {
        let bytes = std::fs::read(local_path).map_err(|e| e.to_string())?;
        let mut lock = self.files.lock().map_err(|e| e.to_string())?;
        lock.insert(remote_filename.to_string(), bytes);
        Ok(())
    }

    async fn download_file(&self, remote_filename: &str, local_path: &Path) -> Result<(), String> {
        let lock = self.files.lock().map_err(|e| e.to_string())?;
        let bytes = lock
            .get(remote_filename)
            .ok_or_else(|| format!("Archivo no encontrado en nube: {}", remote_filename))?;
        std::fs::write(local_path, bytes).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Proveedor de sincronización basado en directorio (MEGAsync, OneDrive, Google Drive Desktop, etc.)
#[derive(Debug, Clone)]
pub struct FolderStorageProvider {
    pub provider_type: CloudProviderType,
    pub cloud_dir: PathBuf,
}

impl FolderStorageProvider {
    pub fn new(provider_type: CloudProviderType, cloud_dir: PathBuf) -> Self {
        Self {
            provider_type,
            cloud_dir,
        }
    }
}

impl CloudStorageProvider for FolderStorageProvider {
    fn provider_type(&self) -> CloudProviderType {
        self.provider_type
    }

    fn is_configured(&self) -> bool {
        !self.cloud_dir.as_os_str().is_empty()
    }

    async fn get_remote_manifest(&self) -> Result<Option<SyncManifest>, String> {
        let manifest_path = self.cloud_dir.join("stickup_manifest.json");
        if !manifest_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        let manifest: SyncManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Some(manifest))
    }

    async fn save_remote_manifest(&self, manifest: &SyncManifest) -> Result<(), String> {
        std::fs::create_dir_all(&self.cloud_dir).map_err(|e| e.to_string())?;
        let manifest_path = self.cloud_dir.join("stickup_manifest.json");
        let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
        std::fs::write(&manifest_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn upload_file(&self, local_path: &Path, remote_filename: &str) -> Result<(), String> {
        std::fs::create_dir_all(&self.cloud_dir).map_err(|e| e.to_string())?;
        let dest = self.cloud_dir.join(remote_filename);
        std::fs::copy(local_path, &dest).map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn download_file(&self, remote_filename: &str, local_path: &Path) -> Result<(), String> {
        let src = self.cloud_dir.join(remote_filename);
        if !src.exists() {
            return Err(format!("Archivo no encontrado en nube: {}", remote_filename));
        }
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&src, local_path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Despachador unificado para cualquier proveedor soportado
#[derive(Clone)]
pub enum AnyCloudProvider {
    Memory(MemoryStorageProvider),
    Folder(FolderStorageProvider),
}

impl CloudStorageProvider for AnyCloudProvider {
    fn provider_type(&self) -> CloudProviderType {
        match self {
            Self::Memory(p) => p.provider_type(),
            Self::Folder(p) => p.provider_type(),
        }
    }

    fn is_configured(&self) -> bool {
        match self {
            Self::Memory(p) => p.is_configured(),
            Self::Folder(p) => p.is_configured(),
        }
    }

    async fn get_remote_manifest(&self) -> Result<Option<SyncManifest>, String> {
        match self {
            Self::Memory(p) => p.get_remote_manifest().await,
            Self::Folder(p) => p.get_remote_manifest().await,
        }
    }

    async fn save_remote_manifest(&self, manifest: &SyncManifest) -> Result<(), String> {
        match self {
            Self::Memory(p) => p.save_remote_manifest(manifest).await,
            Self::Folder(p) => p.save_remote_manifest(manifest).await,
        }
    }

    async fn upload_file(&self, local_path: &Path, remote_filename: &str) -> Result<(), String> {
        match self {
            Self::Memory(p) => p.upload_file(local_path, remote_filename).await,
            Self::Folder(p) => p.upload_file(local_path, remote_filename).await,
        }
    }

    async fn download_file(&self, remote_filename: &str, local_path: &Path) -> Result<(), String> {
        match self {
            Self::Memory(p) => p.download_file(remote_filename, local_path).await,
            Self::Folder(p) => p.download_file(remote_filename, local_path).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_storage_provider_lifecycle() {
        tauri::async_runtime::block_on(async {
            let temp_dir = std::env::temp_dir().join(format!("test_cloud_dir_{}", uuid::Uuid::new_v4()));
            let provider = FolderStorageProvider::new(CloudProviderType::Mega, temp_dir.clone());

            assert_eq!(provider.provider_type(), CloudProviderType::Mega);
            assert!(provider.is_configured());

            // 1. Manifiesto inicial es None
            let init_manifest = provider.get_remote_manifest().await.unwrap();
            assert!(init_manifest.is_none());

            // 2. Guardar manifiesto
            let manifest = SyncManifest {
                version: 1,
                generated_at: 123456789,
                items: HashMap::new(),
            };
            provider.save_remote_manifest(&manifest).await.unwrap();

            // 3. Leer manifiesto guardado
            let loaded = provider.get_remote_manifest().await.unwrap().unwrap();
            assert_eq!(loaded.generated_at, 123456789);

            // 4. Subir y descargar archivo
            let local_test_file = temp_dir.join("local_sample.png");
            std::fs::write(&local_test_file, b"sample bytes").unwrap();
            provider.upload_file(&local_test_file, "remote_sample.png").await.unwrap();

            let target_download = temp_dir.join("downloaded.png");
            provider.download_file("remote_sample.png", &target_download).await.unwrap();
            assert_eq!(std::fs::read(&target_download).unwrap(), b"sample bytes");

            // Limpieza
            let _ = std::fs::remove_dir_all(&temp_dir);
        });
    }
}
