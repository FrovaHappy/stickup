use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Emote {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
    pub path: String,
    pub original_width: i32,
    pub original_height: i32,
    pub format: String,
    pub created_at: String,
    pub file_size: Option<u64>,
    pub copy_count: i64,
    pub interaction_count: i64,
    pub last_used_at: Option<String>,
}

#[derive(Serialize, Default)]
pub struct ResyncStats {
    pub scanned_files: u32,
    pub converted_to_webp: u32,
    pub png_converted: u32,
    pub jpg_converted: u32,
    pub added_to_db: u32,
    pub removed_missing_from_db: u32,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub saved_bytes: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct BackupInfo {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub emote_count: usize,
    pub created_at: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct BackupResult {
    pub success: bool,
    pub backup: BackupInfo,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RestoreResult {
    pub success: bool,
    pub restored_emote_count: usize,
    pub safety_backup_path: Option<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ScannedImage {
    pub url: String,
    pub alt: Option<String>,
    pub suggested_name: String,
    pub format_hint: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScannedImageImportRequest {
    pub url: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BatchImportResult {
    pub imported: Vec<Emote>,
    pub failed_count: usize,
    pub errors: Vec<String>,
}
