use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncItem {
    pub id: i64,
    pub file_name: String,
    pub name: String,
    pub tag: Option<String>,
    pub format: String,
    pub original_width: Option<i32>,
    pub original_height: Option<i32>,
    pub hash: String,
    pub updated_at: i64,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SyncManifest {
    pub version: u32,
    pub generated_at: i64,
    pub items: HashMap<String, SyncItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncDiff {
    pub to_upload: Vec<String>,
    pub to_download: Vec<String>,
    pub apply_to_local: Vec<SyncItem>,
    pub apply_to_remote: Vec<SyncItem>,
}

pub fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Error leyendo archivo para hash {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn calculate_diff(local: &SyncManifest, remote: &SyncManifest) -> SyncDiff {
    let mut diff = SyncDiff::default();

    for (file_name, local_item) in &local.items {
        match remote.items.get(file_name) {
            None => {
                if !local_item.is_deleted {
                    diff.to_upload.push(file_name.clone());
                }
                diff.apply_to_remote.push(local_item.clone());
            }
            Some(remote_item) => {
                if local_item.updated_at > remote_item.updated_at {
                    if local_item.hash != remote_item.hash && !local_item.is_deleted {
                        diff.to_upload.push(file_name.clone());
                    }
                    diff.apply_to_remote.push(local_item.clone());
                } else if remote_item.updated_at > local_item.updated_at {
                    if remote_item.hash != local_item.hash && !remote_item.is_deleted {
                        diff.to_download.push(file_name.clone());
                    }
                    diff.apply_to_local.push(remote_item.clone());
                }
            }
        }
    }

    for (file_name, remote_item) in &remote.items {
        if !local.items.contains_key(file_name) {
            if !remote_item.is_deleted {
                diff.to_download.push(file_name.clone());
            }
            diff.apply_to_local.push(remote_item.clone());
        }
    }

    diff.to_upload.sort();
    diff.to_download.sort();
    diff
}

pub fn build_local_manifest(conn: &Connection, resources_dir: &Path) -> Result<SyncManifest, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, tag, path, original_width, original_height, format, updated_at, is_deleted, hash
             FROM emotes",
        )
        .map_err(|e| e.to_string())?;

    let rows_iter = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            let tag: Option<String> = row.get(2)?;
            let raw_path: String = row.get(3)?;
            let original_width: Option<i32> = row.get(4)?;
            let original_height: Option<i32> = row.get(5)?;
            let format: String = row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "webp".to_string());
            let updated_at: i64 = row.get::<_, Option<i64>>(7)?.unwrap_or(0);
            let is_deleted: bool = row.get::<_, Option<i32>>(8)?.unwrap_or(0) == 1;
            let cached_hash: String = row.get::<_, Option<String>>(9)?.unwrap_or_default();
            Ok((id, name, tag, raw_path, original_width, original_height, format, updated_at, is_deleted, cached_hash))
        })
        .map_err(|e| e.to_string())?;

    let mut items = HashMap::new();
    let mut hashes_to_cache: Vec<(i64, String)> = Vec::new();

    for r in rows_iter {
        let (id, name, tag, raw_path, original_width, original_height, format, updated_at, is_deleted, mut cached_hash) =
            r.map_err(|e| e.to_string())?;

        let clean_path = crate::db::clean_path_str(&raw_path);
        let p = Path::new(&clean_path);
        let file_name = match p.file_name().and_then(|f| f.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };

        let file_path = resources_dir.join(&file_name);

        if cached_hash.is_empty() && file_path.exists() {
            if let Ok(computed) = compute_file_sha256(&file_path) {
                cached_hash = computed.clone();
                hashes_to_cache.push((id, computed));
            }
        }

        items.insert(
            file_name.clone(),
            SyncItem {
                id,
                file_name,
                name,
                tag,
                format,
                original_width,
                original_height,
                hash: cached_hash,
                updated_at,
                is_deleted,
            },
        );
    }

    for (id, hash) in hashes_to_cache {
        let _ = conn.execute("UPDATE emotes SET hash = ?1 WHERE id = ?2", rusqlite::params![hash, id]);
    }

    Ok(SyncManifest {
        version: 1,
        generated_at: chrono::Utc::now().timestamp_millis(),
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_item(file: &str, updated_at: i64, hash: &str, is_deleted: bool) -> SyncItem {
        SyncItem {
            id: 1,
            file_name: file.to_string(),
            name: "test".to_string(),
            tag: Some("funny".to_string()),
            format: "png".to_string(),
            original_width: Some(128),
            original_height: Some(128),
            hash: hash.to_string(),
            updated_at,
            is_deleted,
        }
    }

    #[test]
    fn test_calculate_diff_new_local_item() {
        let mut local = SyncManifest::default();
        let remote = SyncManifest::default();
        local.items.insert("pepe.png".to_string(), dummy_item("pepe.png", 100, "hash1", false));

        let diff = calculate_diff(&local, &remote);
        assert_eq!(diff.to_upload, vec!["pepe.png"]);
        assert_eq!(diff.to_download.len(), 0);
        assert_eq!(diff.apply_to_remote.len(), 1);
        assert_eq!(diff.apply_to_local.len(), 0);
    }

    #[test]
    fn test_calculate_diff_new_remote_item() {
        let local = SyncManifest::default();
        let mut remote = SyncManifest::default();
        remote.items.insert("cat.png".to_string(), dummy_item("cat.png", 100, "hash_cat", false));

        let diff = calculate_diff(&local, &remote);
        assert_eq!(diff.to_upload.len(), 0);
        assert_eq!(diff.to_download, vec!["cat.png"]);
        assert_eq!(diff.apply_to_local.len(), 1);
    }

    #[test]
    fn test_calculate_diff_last_write_wins_local_wins() {
        let mut local = SyncManifest::default();
        let mut remote = SyncManifest::default();
        local.items.insert("dog.png".to_string(), dummy_item("dog.png", 200, "hash_v2", false));
        remote.items.insert("dog.png".to_string(), dummy_item("dog.png", 100, "hash_v1", false));

        let diff = calculate_diff(&local, &remote);
        assert_eq!(diff.to_upload, vec!["dog.png"]);
        assert_eq!(diff.to_download.len(), 0);
        assert_eq!(diff.apply_to_remote.len(), 1);
        assert_eq!(diff.apply_to_local.len(), 0);
    }

    #[test]
    fn test_calculate_diff_last_write_wins_remote_wins() {
        let mut local = SyncManifest::default();
        let mut remote = SyncManifest::default();
        local.items.insert("dog.png".to_string(), dummy_item("dog.png", 100, "hash_v1", false));
        remote.items.insert("dog.png".to_string(), dummy_item("dog.png", 200, "hash_v2", false));

        let diff = calculate_diff(&local, &remote);
        assert_eq!(diff.to_download, vec!["dog.png"]);
        assert_eq!(diff.to_upload.len(), 0);
        assert_eq!(diff.apply_to_local.len(), 1);
        assert_eq!(diff.apply_to_remote.len(), 0);
    }

    #[test]
    fn test_calculate_diff_deleted_propagation() {
        let mut local = SyncManifest::default();
        let mut remote = SyncManifest::default();
        local.items.insert("gone.png".to_string(), dummy_item("gone.png", 300, "hash", true));
        remote.items.insert("gone.png".to_string(), dummy_item("gone.png", 100, "hash", false));

        let diff = calculate_diff(&local, &remote);
        assert_eq!(diff.to_upload.len(), 0);
        assert_eq!(diff.to_download.len(), 0);
        assert_eq!(diff.apply_to_remote.len(), 1);
        assert!(diff.apply_to_remote[0].is_deleted);
    }

    #[test]
    fn test_compute_file_sha256() {
        let tmp = std::env::temp_dir().join(format!("test_sha_{}", uuid::Uuid::new_v4()));
        fs::write(&tmp, b"hello stickup").unwrap();
        let hash = compute_file_sha256(&tmp).unwrap();
        assert_eq!(hash, "66d15df5a30179e44c14801b8a9b68b51d5e3887f3bc43c6ac6a626abd8063fc");
        let _ = fs::remove_file(&tmp);
    }
}
