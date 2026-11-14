use super::manifest::{build_local_manifest, calculate_diff, SyncItem, SyncManifest};
use super::provider::{
    AnyCloudProvider, CloudProviderType, CloudStorageProvider, FolderStorageProvider, SyncStatus,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static SYNC_STATUS: OnceLock<Mutex<SyncStatus>> = OnceLock::new();

pub fn get_current_sync_status() -> SyncStatus {
    SYNC_STATUS
        .get_or_init(|| Mutex::new(SyncStatus::default()))
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}

pub fn update_current_sync_status<F: FnOnce(&mut SyncStatus)>(update_fn: F) {
    if let Ok(mut lock) = SYNC_STATUS
        .get_or_init(|| Mutex::new(SyncStatus::default()))
        .lock()
    {
        update_fn(&mut lock);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub provider: String,
    pub cloud_path: Option<String>,
    pub auto_sync: bool,
    pub last_sync_at: Option<i64>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: "none".to_string(),
            cloud_path: None,
            auto_sync: false,
            last_sync_at: None,
        }
    }
}

pub fn get_sync_config(conn: &Connection) -> Result<SyncConfig, String> {
    let provider: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'sync_provider'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "none".to_string());

    let cloud_path: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'sync_cloud_path'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let auto_sync: bool = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'sync_auto_enabled'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .map(|v: String| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let last_sync_at: Option<i64> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'sync_last_at'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .and_then(|v: String| v.parse::<i64>().ok());

    Ok(SyncConfig {
        provider,
        cloud_path,
        auto_sync,
        last_sync_at,
    })
}

pub fn save_sync_config(conn: &Connection, config: &SyncConfig) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_provider', ?1)",
        params![config.provider],
    )
    .map_err(|e| e.to_string())?;

    if let Some(ref path) = config.cloud_path {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_cloud_path', ?1)",
            params![path],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute("DELETE FROM settings WHERE key = 'sync_cloud_path'", [])
            .map_err(|e| e.to_string())?;
    }

    let auto_val = if config.auto_sync { "1" } else { "0" };
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_auto_enabled', ?1)",
        params![auto_val],
    )
    .map_err(|e| e.to_string())?;

    if let Some(last_at) = config.last_sync_at {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_last_at', ?1)",
            params![last_at.to_string()],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn detect_cloud_candidates() -> HashMap<String, String> {
    let mut candidates = HashMap::new();

    #[cfg(windows)]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let user_profile = PathBuf::from(profile);
            let mega_candidates = [
                user_profile.join("Documents").join("MEGA").join("MEGAsync"),
                user_profile.join("Documentos").join("MEGA").join("MEGAsync"),
                user_profile.join("Documents").join("MEGAsync"),
                user_profile.join("MEGAsync"),
                user_profile.join("MEGA"),
            ];
            for path in mega_candidates {
                if path.is_dir() {
                    candidates.insert(
                        "mega".to_string(),
                        path.join("StickUp").to_string_lossy().to_string(),
                    );
                    break;
                }
            }

            if let Ok(onedrive) = std::env::var("OneDrive") {
                let onedrive_path = PathBuf::from(onedrive);
                if onedrive_path.is_dir() {
                    candidates.insert(
                        "onedrive".to_string(),
                        onedrive_path.join("StickUp").to_string_lossy().to_string(),
                    );
                }
            } else if user_profile.join("OneDrive").is_dir() {
                candidates.insert(
                    "onedrive".to_string(),
                    user_profile
                        .join("OneDrive")
                        .join("StickUp")
                        .to_string_lossy()
                        .to_string(),
                );
            }

            let gdrive_paths = [
                PathBuf::from(r"G:\Mi unidad"),
                PathBuf::from(r"G:\My Drive"),
                user_profile.join("Google Drive"),
            ];
            for path in gdrive_paths {
                if path.is_dir() {
                    candidates.insert(
                        "googledrive".to_string(),
                        path.join("StickUp").to_string_lossy().to_string(),
                    );
                    break;
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            if home_path.join("MEGAsync").is_dir() {
                candidates.insert(
                    "mega".to_string(),
                    home_path.join("MEGAsync").join("StickUp").to_string_lossy().to_string(),
                );
            }
            if home_path.join("OneDrive").is_dir() {
                candidates.insert(
                    "onedrive".to_string(),
                    home_path.join("OneDrive").join("StickUp").to_string_lossy().to_string(),
                );
            }
            if home_path.join("GoogleDrive").is_dir() {
                candidates.insert(
                    "googledrive".to_string(),
                    home_path.join("GoogleDrive").join("StickUp").to_string_lossy().to_string(),
                );
            }
        }
    }

    candidates
}

pub fn resolve_configured_provider(config: &SyncConfig) -> Result<AnyCloudProvider, String> {
    let p_type = CloudProviderType::from_str_opt(&config.provider)
        .ok_or_else(|| format!("Proveedor desconocido o desactivado: {}", config.provider))?;

    let cloud_path_str = config
        .cloud_path
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "No se ha configurado la ruta de la carpeta en la nube".to_string())?;

    let cloud_dir = PathBuf::from(cloud_path_str);
    Ok(AnyCloudProvider::Folder(FolderStorageProvider::new(
        p_type, cloud_dir,
    )))
}

pub async fn execute_sync<P: CloudStorageProvider>(
    provider: &P,
    resources_dir: &Path,
) -> Result<SyncStatus, String> {
    execute_sync_with_db_opener(provider, resources_dir, || crate::db::ensure_db()).await
}

pub async fn execute_sync_with_db_opener<P, F>(
    provider: &P,
    resources_dir: &Path,
    open_conn: F,
) -> Result<SyncStatus, String>
where
    P: CloudStorageProvider,
    F: Fn() -> Result<Connection, String> + Send,
{
    // 1. Construir manifiesto local (conexión se usa y se libera antes del primer .await)
    let local_manifest = {
        let conn = open_conn()?;
        build_local_manifest(&conn, resources_dir)?
    };

    // 2. Obtener manifiesto remoto (o inicializarlo vacío si es la primera vez)
    let remote_manifest = provider
        .get_remote_manifest()
        .await?
        .unwrap_or_default();

    // 3. Calcular diferencias y deltas
    let diff = calculate_diff(&local_manifest, &remote_manifest);
    let uploaded_count = diff.to_upload.len() as u32;
    let downloaded_count = diff.to_download.len() as u32;

    // 4. Subir archivos binarios faltantes en la nube
    for file_name in &diff.to_upload {
        let local_path = resources_dir.join(file_name);
        if local_path.exists() {
            provider.upload_file(&local_path, file_name).await?;
        }
    }

    // 5. Descargar archivos binarios faltantes localmente
    for file_name in &diff.to_download {
        let local_path = resources_dir.join(file_name);
        provider.download_file(file_name, &local_path).await?;
    }

    // 6. Aplicar cambios a la base de datos local SQLite (conexión se usa y se libera antes del .await)
    if !diff.apply_to_local.is_empty() {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for item in &diff.apply_to_local {
            apply_item_to_local_db(&tx, resources_dir, item)?;
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    // 7. Actualizar el manifiesto remoto con los deltas locales
    let mut updated_remote_items: HashMap<String, SyncItem> = remote_manifest.items;
    for item in diff.apply_to_remote {
        updated_remote_items.insert(item.file_name.clone(), item);
    }
    for item in &diff.apply_to_local {
        updated_remote_items.insert(item.file_name.clone(), item.clone());
    }

    let now = chrono::Utc::now().timestamp_millis();
    let new_remote_manifest = SyncManifest {
        version: 1,
        generated_at: now,
        items: updated_remote_items,
    };
    provider.save_remote_manifest(&new_remote_manifest).await?;

    let status = SyncStatus {
        is_syncing: false,
        last_sync_at: Some(now),
        provider: Some(provider.provider_type().as_str().to_string()),
        progress: Some(format!(
            "Sincronizado: {} subidos, {} descargados",
            uploaded_count, downloaded_count
        )),
        error: None,
        uploaded_count,
        downloaded_count,
    };

    Ok(status)
}

fn apply_item_to_local_db(
    conn: &Connection,
    resources_dir: &Path,
    item: &SyncItem,
) -> Result<(), String> {
    let local_file = resources_dir.join(&item.file_name);
    let norm_path = crate::db::normalize_path(&local_file);
    let is_del_int = if item.is_deleted { 1 } else { 0 };

    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM emotes WHERE path = ?1 OR path LIKE ?2",
            params![norm_path, format!("%/{}", item.file_name)],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE emotes
             SET name = ?1, tag = ?2, format = ?3, original_width = ?4, original_height = ?5,
                 updated_at = ?6, is_deleted = ?7, hash = ?8
             WHERE id = ?9",
            params![
                item.name,
                item.tag,
                item.format,
                item.original_width,
                item.original_height,
                item.updated_at,
                is_del_int,
                item.hash,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
    } else {
        let now_str = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO emotes (name, tag, path, original_width, original_height, format, created_at, updated_at, is_deleted, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                item.name,
                item.tag,
                norm_path,
                item.original_width,
                item.original_height,
                item.format,
                now_str,
                item.updated_at,
                is_del_int,
                item.hash
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::sync::provider::MemoryStorageProvider;

    #[test]
    fn test_sync_config_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);")
            .unwrap();

        let cfg = SyncConfig {
            provider: "mega".to_string(),
            cloud_path: Some("C:\\Cloud\\StickUp".to_string()),
            auto_sync: true,
            last_sync_at: Some(1700000000),
        };

        save_sync_config(&conn, &cfg).unwrap();
        let loaded = get_sync_config(&conn).unwrap();

        assert_eq!(loaded.provider, "mega");
        assert_eq!(loaded.cloud_path.as_deref(), Some("C:\\Cloud\\StickUp"));
        assert!(loaded.auto_sync);
        assert_eq!(loaded.last_sync_at, Some(1700000000));
    }

    #[test]
    fn test_execute_sync_uploads_and_downloads_with_memory_provider() {
        tauri::async_runtime::block_on(async {
            let temp_dir =
                std::env::temp_dir().join(format!("test_sync_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&temp_dir).unwrap();

            let db_path = temp_dir.join("test.db");
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE emotes (
                    id INTEGER PRIMARY KEY,
                    name TEXT,
                    tag TEXT,
                    path TEXT NOT NULL,
                    original_width INTEGER,
                    original_height INTEGER,
                    format TEXT,
                    created_at TEXT,
                    updated_at INTEGER DEFAULT 0,
                    is_deleted INTEGER DEFAULT 0,
                    hash TEXT
                );",
            )
            .unwrap();

            // Crear archivo local
            let local_file = temp_dir.join("smile.png");
            std::fs::write(&local_file, b"smile_image_content").unwrap();
            conn.execute(
                "INSERT INTO emotes (name, tag, path, format, created_at, updated_at, is_deleted)
                 VALUES ('smile', 'happy', ?1, 'png', '2026-11-13', 100, 0)",
                params![crate::db::normalize_path(&local_file)],
            )
            .unwrap();

            let provider = MemoryStorageProvider::default();

            // 1er sync: debe subir smile.png a la nube
            let db_path_c1 = db_path.clone();
            let status = execute_sync_with_db_opener(&provider, &temp_dir, move || {
                Connection::open(&db_path_c1).map_err(|e| e.to_string())
            })
            .await
            .unwrap();
            assert_eq!(status.uploaded_count, 1);
            assert_eq!(status.downloaded_count, 0);

            // Validar que el archivo está en la memoria del provider
            let files_lock = provider.files.lock().unwrap();
            assert!(files_lock.contains_key("smile.png"));
            drop(files_lock);

            // Simular que en la nube otro dispositivo subió "party.png"
            let mut remote_manifest = provider.get_remote_manifest().await.unwrap().unwrap();
            remote_manifest.items.insert(
                "party.png".to_string(),
                SyncItem {
                    id: 99,
                    file_name: "party.png".to_string(),
                    name: "party".to_string(),
                    tag: Some("fun".to_string()),
                    format: "png".to_string(),
                    original_width: Some(100),
                    original_height: Some(100),
                    hash: "hash_party".to_string(),
                    updated_at: 500,
                    is_deleted: false,
                },
            );
            provider.save_remote_manifest(&remote_manifest).await.unwrap();
            provider
                .files
                .lock()
                .unwrap()
                .insert("party.png".to_string(), b"party_bytes".to_vec());

            // 2do sync: debe descargar "party.png" y agregarlo a SQLite
            let db_path_c2 = db_path.clone();
            let status2 = execute_sync_with_db_opener(&provider, &temp_dir, move || {
                Connection::open(&db_path_c2).map_err(|e| e.to_string())
            })
            .await
            .unwrap();
            assert_eq!(status2.uploaded_count, 0);
            assert_eq!(status2.downloaded_count, 1);

            // Verificar que el archivo fue descargado en disco local
            let downloaded_path = temp_dir.join("party.png");
            assert!(downloaded_path.exists());
            assert_eq!(std::fs::read(&downloaded_path).unwrap(), b"party_bytes");

            // Verificar que SQLite contiene el emote descargado
            let party_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM emotes WHERE name = 'party'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(party_count, 1);

            let _ = std::fs::remove_dir_all(&temp_dir);
        });
    }
}
