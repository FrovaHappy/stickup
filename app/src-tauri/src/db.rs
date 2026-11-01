use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

pub fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s
    }
}

pub fn clean_path_str(raw_path: String) -> String {
    if let Some(stripped) = raw_path.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        raw_path
    }
}

fn db_path() -> PathBuf {
    if let Ok(current) = std::env::current_dir() {
        if current.ends_with("src-tauri") {
            if let Some(parent) = current.parent() {
                return parent.join("emotes.db");
            }
        }
        return current.join("emotes.db");
    }
    PathBuf::from("emotes.db")
}

fn get_default_emotes_dir() -> PathBuf {
    if let Ok(current) = std::env::current_dir() {
        if current.ends_with("src-tauri") {
            if let Some(parent) = current.parent() {
                return parent.join("emotes");
            }
        }
        return current.join("emotes");
    }
    PathBuf::from("emotes")
}

pub fn get_active_emotes_dir(conn: &Connection) -> PathBuf {
    if let Ok(mut stmt) = conn.prepare("SELECT value FROM settings WHERE key = 'storage_path'") {
        if let Ok(path_str) = stmt.query_row([], |row| row.get::<_, String>(0)) {
            let clean = clean_path_str(path_str);
            if !clean.contains("src-tauri") {
                let p = PathBuf::from(clean);
                if !p.as_os_str().is_empty() {
                    return p;
                }
            }
        }
    }
    let default_dir = get_default_emotes_dir();
    let norm = normalize_path(&default_dir);
    let _ = conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('storage_path', ?1)",
        params![norm],
    );
    default_dir
}

pub fn ensure_db() -> Result<Connection, String> {
    let db = db_path();
    if let Some(parent) = db.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let conn = Connection::open(&db).map_err(|e| format!("{}: {}", e, db.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS emotes (
            id INTEGER PRIMARY KEY,
            name TEXT,
            tag TEXT,
            path TEXT NOT NULL,
            original_width INTEGER,
            original_height INTEGER,
            format TEXT,
            created_at TEXT
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );",
    )
    .map_err(|e| e.to_string())?;

    let _ = conn.execute(
        "UPDATE emotes SET path = SUBSTR(path, 5) WHERE path LIKE '\\\\?\\%'",
        [],
    );

    Ok(conn)
}

