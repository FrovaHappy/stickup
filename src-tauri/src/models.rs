use serde::Serialize;

#[derive(Serialize)]
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
