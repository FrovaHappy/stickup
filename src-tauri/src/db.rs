use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

pub const DB_FILE_NAME: &str = ".store.db";
const LEGACY_DB_FILE_NAME: &str = "emotes.db";

pub fn clean_path_str(raw_path: &str) -> String {
    raw_path.strip_prefix(r"\\?\").unwrap_or(raw_path).to_string()
}

pub fn normalize_path(path: &Path) -> String {
    clean_path_str(&path.to_string_lossy())
}

fn project_root() -> PathBuf {
    if let Ok(current) = std::env::current_dir() {
        if current.ends_with("src-tauri") {
            if let Some(parent) = current.parent() {
                return parent.to_path_buf();
            }
        }
        return current;
    }
    PathBuf::new()
}

fn os_documents_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let docs = PathBuf::from(&profile).join("Documents");
            if docs.exists() {
                return Some(docs);
            }
            if let Ok(onedrive) = std::env::var("OneDrive") {
                let od_docs = PathBuf::from(onedrive).join("Documents");
                if od_docs.exists() {
                    return Some(od_docs);
                }
            }
            return Some(docs);
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(xdg) = std::env::var("XDG_DOCUMENTS_DIR") {
            let trimmed = xdg.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            let docs = home_path.join("Documents");
            if docs.exists() {
                return Some(docs);
            }
            let docs_es = home_path.join("Documentos");
            if docs_es.exists() {
                return Some(docs_es);
            }
            return Some(docs);
        }
    }
    None
}

fn default_emotes_dir() -> PathBuf {
    os_documents_dir()
        .unwrap_or_else(project_root)
        .join("emotes")
}

fn db_dir_config_file() -> PathBuf {
    project_root().join(".stickup_db_dir")
}

pub fn upgrade_legacy_db_in_dir(dir: &Path) {
    let store_db = dir.join(DB_FILE_NAME);
    let legacy_db = dir.join(LEGACY_DB_FILE_NAME);
    if !store_db.exists() && legacy_db.exists() {
        if fs::rename(&legacy_db, &store_db).is_err() && fs::copy(&legacy_db, &store_db).is_ok() {
            let _ = fs::remove_file(&legacy_db);
        }
    }
}

pub fn get_active_db_dir() -> PathBuf {
    let cfg = db_dir_config_file();
    if let Ok(raw) = fs::read_to_string(&cfg) {
        let clean = clean_path_str(raw.trim());
        if !clean.is_empty() && !clean.contains("src-tauri") {
            return PathBuf::from(clean);
        }
    }
    let default_dir = default_emotes_dir();
    if !default_dir.exists() {
        let _ = fs::create_dir_all(&default_dir);
    }
    let abs = default_dir.canonicalize().unwrap_or(default_dir);
    let abs_str = normalize_path(&abs);
    let _ = fs::write(&cfg, &abs_str);
    abs
}

pub fn set_active_db_dir(path: &str, migrate: bool) -> Result<String, String> {
    let p = PathBuf::from(path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(|e| format!("No se pudo crear la carpeta de BD: {}", e))?;
    }
    let abs = p.canonicalize().unwrap_or(p);
    let abs_str = normalize_path(&abs);

    if abs_str.contains("src-tauri") {
        return Err(
            "No se puede ubicar la base de datos dentro de src-tauri (provocaría recompilaciones)"
                .to_string(),
        );
    }

    let old_dir = get_active_db_dir();
    upgrade_legacy_db_in_dir(&old_dir);
    upgrade_legacy_db_in_dir(&abs);

    let old_db = old_dir.join(DB_FILE_NAME);
    let new_db = abs.join(DB_FILE_NAME);

    if migrate && old_db != new_db && old_db.exists() {
        fs::copy(&old_db, &new_db)
            .map_err(|e| format!("No se pudo migrar {} a la nueva ruta: {}", DB_FILE_NAME, e))?;
        let _ = fs::remove_file(&old_db);
    }

    fs::write(db_dir_config_file(), &abs_str)
        .map_err(|e| format!("No se pudo guardar la configuración de ruta de BD: {}", e))?;

    let _ = ensure_db()?;
    Ok(abs_str)
}

pub fn get_active_emotes_dir(conn: &Connection) -> PathBuf {
    if let Ok(mut stmt) = conn.prepare("SELECT value FROM settings WHERE key = 'storage_path'") {
        if let Ok(path_str) = stmt.query_row([], |row| row.get::<_, String>(0)) {
            let clean = clean_path_str(&path_str);
            if !clean.contains("src-tauri") && !clean.is_empty() {
                return PathBuf::from(clean);
            }
        }
    }
    let default_dir = get_active_db_dir();
    let norm = normalize_path(&default_dir);
    let _ = conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('storage_path', ?1)",
        params![norm],
    );
    default_dir
}

pub fn ensure_db() -> Result<Connection, String> {
    let db_dir = get_active_db_dir();
    if !db_dir.as_os_str().is_empty() && !db_dir.exists() {
        let _ = fs::create_dir_all(&db_dir);
    }
    upgrade_legacy_db_in_dir(&db_dir);
    let db = db_dir.join(DB_FILE_NAME);
    let legacy_root_db = project_root().join(LEGACY_DB_FILE_NAME);
    if legacy_root_db != db && legacy_root_db.exists() && !db.exists() {
        if fs::copy(&legacy_root_db, &db).is_ok() {
            let _ = fs::remove_file(&legacy_root_db);
        }
    }
    let conn = Connection::open(&db).map_err(|e| format!("{}: {}", e, db.display()))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS emotes (
            id INTEGER PRIMARY KEY,
            name TEXT,
            tag TEXT,
            path TEXT NOT NULL,
            original_width INTEGER,
            original_height INTEGER,
            format TEXT,
            created_at TEXT,
            copy_count INTEGER DEFAULT 0,
            interaction_count INTEGER DEFAULT 0,
            last_used_at TEXT
        );
        CREATE TABLE IF NOT EXISTS emote_votes (
            emote_id INTEGER PRIMARY KEY,
            copy_count INTEGER NOT NULL DEFAULT 0,
            interaction_count INTEGER NOT NULL DEFAULT 0,
            score INTEGER NOT NULL DEFAULT 0,
            last_used_at TEXT,
            FOREIGN KEY (emote_id) REFERENCES emotes(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );",
    )
    .map_err(|e| e.to_string())?;

    let _ = conn.execute("ALTER TABLE emotes ADD COLUMN copy_count INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE emotes ADD COLUMN interaction_count INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE emotes ADD COLUMN last_used_at TEXT", []);
    let _ = conn.execute(
        "UPDATE emotes SET path = SUBSTR(path, 5) WHERE path LIKE '\\\\?\\%'",
        [],
    );

    // Migrar datos históricos de copias desde emotes hacia la tabla dedicada emote_votes
    let _ = conn.execute(
        "INSERT OR IGNORE INTO emote_votes (emote_id, copy_count, interaction_count, score, last_used_at)
         SELECT id,
                COALESCE(copy_count, 0),
                COALESCE(interaction_count, 0),
                COALESCE(copy_count, 0),
                last_used_at
         FROM emotes
         WHERE COALESCE(copy_count, 0) > 0 OR COALESCE(interaction_count, 0) > 0",
        [],
    );
    let _ = conn.execute(
        "UPDATE emote_votes SET score = copy_count WHERE score != copy_count",
        [],
    );
    let _ = conn.execute(
        "DELETE FROM emote_votes WHERE emote_id NOT IN (SELECT id FROM emotes)",
        [],
    );

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_path_str_strips_unc_prefix() {
        assert_eq!(
            clean_path_str(r"\\?\C:\Users\happy\emotes\pepe.webp"),
            r"C:\Users\happy\emotes\pepe.webp"
        );
        assert_eq!(
            clean_path_str("/home/user/emotes/pepe.webp"),
            "/home/user/emotes/pepe.webp"
        );
    }

    #[test]
    fn test_normalize_path() {
        let p = Path::new(r"\\?\C:\emotes\test.gif");
        assert_eq!(normalize_path(p), r"C:\emotes\test.gif");
    }

    #[test]
    fn test_rejects_src_tauri_db_dir() {
        let tmp = std::env::temp_dir().join("src-tauri").join("test_db_guard");
        let res = set_active_db_dir(&tmp.to_string_lossy(), false);
        assert!(res.is_err());
        let _ = fs::remove_dir_all(&tmp);
    }
}
