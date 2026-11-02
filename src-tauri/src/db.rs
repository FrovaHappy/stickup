use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

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

fn db_dir_config_file() -> PathBuf {
    project_root().join(".stickup_db_dir")
}

pub fn get_active_db_dir() -> PathBuf {
    if let Ok(raw) = fs::read_to_string(db_dir_config_file()) {
        let clean = clean_path_str(raw.trim());
        if !clean.is_empty() && !clean.contains("src-tauri") {
            return PathBuf::from(clean);
        }
    }
    project_root().join("emotes")
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

    let old_db = get_active_db_dir().join("emotes.db");
    let new_db = abs.join("emotes.db");

    if migrate && old_db != new_db && old_db.exists() {
        fs::copy(&old_db, &new_db)
            .map_err(|e| format!("No se pudo migrar emotes.db a la nueva ruta: {}", e))?;
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
    let default_dir = project_root().join("emotes");
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
    let db = db_dir.join("emotes.db");
    let legacy_db = project_root().join("emotes.db");
    if legacy_db != db && legacy_db.exists() && !db.exists() {
        if fs::copy(&legacy_db, &db).is_ok() {
            let _ = fs::remove_file(&legacy_db);
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
