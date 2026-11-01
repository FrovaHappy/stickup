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
}

