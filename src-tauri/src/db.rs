use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

pub const DB_FILE_NAME: &str = ".store.db";
const LEGACY_DB_FILE_NAME: &str = "emotes.db";
pub const CURRENT_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAX_LOCAL_BACKUPS: usize = 15;

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
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );",
    )
    .map_err(|e| e.to_string())?;

    // Verificación de versión y backup automático preventivo previo a migraciones
    let _ = check_and_backup_on_version_change(&conn);

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS emotes (
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

pub fn get_recorded_app_version(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'app_version'",
        [],
        |r| r.get(0),
    )
    .ok()
}

pub fn record_app_version(conn: &Connection, version: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('app_version', ?1)",
        params![version],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_minimize_to_tray(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'minimize_to_tray'",
        [],
        |r| r.get::<_, String>(0),
    )
    .map(|v| v == "true" || v == "1")
    .unwrap_or(false)
}

pub fn set_minimize_to_tray(conn: &Connection, enabled: bool) -> Result<(), String> {
    let val = if enabled { "true" } else { "false" };
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('minimize_to_tray', ?1)",
        params![val],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_minimize_on_copy(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'minimize_on_copy'",
        [],
        |r| r.get::<_, String>(0),
    )
    .map(|v| v == "true" || v == "1")
    .unwrap_or(false)
}

pub fn set_minimize_on_copy(conn: &Connection, enabled: bool) -> Result<(), String> {
    let val = if enabled { "true" } else { "false" };
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('minimize_on_copy', ?1)",
        params![val],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_force_gif_loop(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'force_gif_loop'",
        [],
        |r| r.get::<_, String>(0),
    )
    .map(|v| v == "true" || v == "1")
    .unwrap_or(true)
}

pub fn set_force_gif_loop(conn: &Connection, enabled: bool) -> Result<(), String> {
    let val = if enabled { "true" } else { "false" };
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('force_gif_loop', ?1)",
        params![val],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub const DEFAULT_GLOBAL_SHORTCUT: &str = "Super+Control+Period";

pub fn get_global_shortcut(conn: &Connection) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'global_shortcut'",
        [],
        |r| r.get::<_, String>(0),
    )
    .unwrap_or_else(|_| DEFAULT_GLOBAL_SHORTCUT.to_string())
}

pub fn set_global_shortcut(conn: &Connection, shortcut: &str) -> Result<(), String> {
    let val = shortcut.trim();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('global_shortcut', ?1)",
        params![val],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn check_and_backup_version_in_dir(
    conn: &Connection,
    current_version: &str,
    backups_dir: &Path,
) -> Result<Option<crate::models::BackupInfo>, String> {
    let recorded = get_recorded_app_version(conn);

    match recorded {
        Some(old_version) => {
            if old_version != current_version {
                if !backups_dir.exists() {
                    let _ = fs::create_dir_all(backups_dir);
                }
                let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                let clean_old = old_version.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                let clean_cur = current_version.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                let backup_file = backups_dir.join(format!(
                    "pre_upgrade_v{}_to_v{}_{}.db",
                    clean_old, clean_cur, timestamp
                ));

                let backup_info = create_backup_from_conn(conn, &backup_file)?;
                let _ = prune_backups_dir(backups_dir, MAX_LOCAL_BACKUPS);
                record_app_version(conn, current_version)?;
                return Ok(Some(backup_info));
            }
        }
        None => {
            let emote_count = count_emotes_in_connection(conn).unwrap_or(0);
            let mut backup_info = None;
            if emote_count > 0 {
                if !backups_dir.exists() {
                    let _ = fs::create_dir_all(backups_dir);
                }
                let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                let clean_cur = current_version.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                let backup_file = backups_dir.join(format!(
                    "pre_upgrade_legacy_to_v{}_{}.db",
                    clean_cur, timestamp
                ));
                backup_info = create_backup_from_conn(conn, &backup_file).ok();
                let _ = prune_backups_dir(backups_dir, MAX_LOCAL_BACKUPS);
            }

            record_app_version(conn, current_version)?;
            return Ok(backup_info);
        }
    }

    Ok(None)
}

pub fn check_and_backup_on_version_change(
    conn: &Connection,
) -> Result<Option<crate::models::BackupInfo>, String> {
    let backups_dir = get_backups_dir();
    check_and_backup_version_in_dir(conn, CURRENT_APP_VERSION, &backups_dir)
}

pub fn get_backups_dir() -> PathBuf {
    get_active_db_dir().join("backups")
}

pub fn count_emotes_in_connection(conn: &Connection) -> Result<usize, String> {
    let count: i64 = conn
        .query_row("SELECT count(*) FROM emotes", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count as usize)
}

pub fn inspect_backup_file(path: &Path) -> Result<crate::models::BackupInfo, String> {
    if !path.exists() {
        return Err(format!("El archivo de copia no existe: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("La ruta indicada no es un archivo: {}", path.display()));
    }

    let conn = Connection::open(path)
        .map_err(|e| format!("No se pudo abrir el archivo SQLite: {}", e))?;

    let table_exists: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'emotes'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Error al verificar esquema de la base de datos: {}", e))?;

    if table_exists == 0 {
        return Err("El archivo no es una copia de seguridad válida de StickUp (no contiene la tabla 'emotes')".to_string());
    }

    let emote_count = count_emotes_in_connection(&conn)?;

    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let size_bytes = metadata.len();

    let created_at = if let Ok(modified) = metadata.modified() {
        let dt: chrono::DateTime<chrono::Local> = modified.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    };

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "backup.db".to_string());

    Ok(crate::models::BackupInfo {
        file_name,
        path: normalize_path(path),
        size_bytes,
        emote_count,
        created_at,
    })
}

pub fn create_backup_from_conn(
    conn: &Connection,
    target_path: &Path,
) -> Result<crate::models::BackupInfo, String> {
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("No se pudo crear el directorio de destino: {}", e))?;
        }
    }

    let norm_target = normalize_path(target_path);

    if target_path.exists() {
        fs::remove_file(target_path)
            .map_err(|e| format!("No se pudo sobrescribir el archivo de destino: {}", e))?;
    }

    conn.execute("VACUUM INTO ?1", params![norm_target])
        .map_err(|e| format!("Error al generar la copia de seguridad: {}", e))?;

    inspect_backup_file(target_path)
}

pub fn create_db_backup(destination_file: Option<&Path>) -> Result<crate::models::BackupInfo, String> {
    let conn = ensure_db()?;
    let backups_dir = get_backups_dir();
    let target_path = match destination_file {
        Some(p) => p.to_path_buf(),
        None => {
            if !backups_dir.exists() {
                fs::create_dir_all(&backups_dir)
                    .map_err(|e| format!("No se pudo crear carpeta de backups: {}", e))?;
            }
            let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            backups_dir.join(format!("stickup_backup_{}.db", timestamp))
        }
    };
    let info = create_backup_from_conn(&conn, &target_path)?;
    if target_path.starts_with(&backups_dir) {
        let _ = prune_backups_dir(&backups_dir, MAX_LOCAL_BACKUPS);
    }
    Ok(info)
}

pub fn prune_backups_dir(backups_dir: &Path, max_allowed: usize) -> Result<usize, String> {
    if !backups_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(backups_dir)
        .map_err(|e| format!("No se pudo leer la carpeta de backups: {}", e))?;

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            let is_db = p
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"))
                .unwrap_or(false);

            if is_db {
                let mod_time = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files.push((p, mod_time));
            }
        }
    }

    // Ordenar de más reciente a más antiguo
    files.sort_by(|a, b| b.1.cmp(&a.1));

    let mut deleted_count = 0;
    if files.len() > max_allowed {
        for (file_to_remove, _) in files.iter().skip(max_allowed) {
            if fs::remove_file(file_to_remove).is_ok() {
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}

pub fn delete_db_backup(backup_path: &Path) -> Result<bool, String> {
    if !backup_path.exists() {
        return Err(format!("El archivo no existe: {}", backup_path.display()));
    }

    if !backup_path.is_file() {
        return Err("La ruta especificada no es un archivo".to_string());
    }

    let is_db = backup_path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"))
        .unwrap_or(false);

    if !is_db {
        return Err("Solo se pueden eliminar archivos de copia de seguridad (.db / .sqlite)".to_string());
    }

    fs::remove_file(backup_path)
        .map_err(|e| format!("No se pudo eliminar la copia de seguridad: {}", e))?;

    Ok(true)
}

pub fn list_db_backups() -> Result<Vec<crate::models::BackupInfo>, String> {
    let backups_dir = get_backups_dir();
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&backups_dir)
        .map_err(|e| format!("No se pudo leer la carpeta de backups: {}", e))?;

    let mut list = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            let is_db = p
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"))
                .unwrap_or(false);

            if is_db {
                if let Ok(info) = inspect_backup_file(&p) {
                    list.push(info);
                }
            }
        }
    }

    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(list)
}

pub fn restore_db_backup(backup_path: &Path) -> Result<crate::models::RestoreResult, String> {
    let backup_info = inspect_backup_file(backup_path)?;

    let active_db_dir = get_active_db_dir();
    let active_db = active_db_dir.join(DB_FILE_NAME);

    let mut safety_path_str: Option<String> = None;

    if active_db.exists() {
        let backups_dir = get_backups_dir();
        if !backups_dir.exists() {
            let _ = fs::create_dir_all(&backups_dir);
        }
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let safety_file = backups_dir.join(format!("pre_restore_safety_{}.db", timestamp));
        if fs::copy(&active_db, &safety_file).is_ok() {
            safety_path_str = Some(normalize_path(&safety_file));
            let _ = prune_backups_dir(&backups_dir, MAX_LOCAL_BACKUPS);
        }
    }

    fs::copy(backup_path, &active_db)
        .map_err(|e| format!("No se pudo restaurar el archivo de base de datos: {}", e))?;

    let conn = ensure_db()?;
    let restored_count = count_emotes_in_connection(&conn).unwrap_or(backup_info.emote_count);

    Ok(crate::models::RestoreResult {
        success: true,
        restored_emote_count: restored_count,
        safety_backup_path: safety_path_str,
        message: format!(
            "Base de datos restaurada correctamente ({} emotes)",
            restored_count
        ),
    })
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

    #[test]
    fn test_create_and_inspect_backup() {
        let tmp_dir = std::env::temp_dir().join(format!("test_backup_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).unwrap();

        let db_path = tmp_dir.join("source.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE emotes (id INTEGER PRIMARY KEY, name TEXT, path TEXT);
             INSERT INTO emotes (id, name, path) VALUES (1, 'pepe', 'C:/pepe.webp');
             INSERT INTO emotes (id, name, path) VALUES (2, 'cat', 'C:/cat.webp');",
        )
        .unwrap();

        assert_eq!(count_emotes_in_connection(&conn).unwrap(), 2);

        let backup_file = tmp_dir.join("subfolder").join("my_backup.db");
        let info = create_backup_from_conn(&conn, &backup_file).unwrap();

        assert_eq!(info.emote_count, 2);
        assert_eq!(info.file_name, "my_backup.db");
        assert!(info.size_bytes > 0);
        assert!(backup_file.exists());

        let inspected = inspect_backup_file(&backup_file).unwrap();
        assert_eq!(inspected.emote_count, 2);
        assert_eq!(inspected.file_name, "my_backup.db");

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_inspect_backup_file_rejects_invalid_database() {
        let tmp_dir = std::env::temp_dir().join(format!("test_invalid_db_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).unwrap();

        // 1. Archivo que no es SQLite
        let text_file = tmp_dir.join("fake.db");
        fs::write(&text_file, "este no es un archivo sqlite").unwrap();
        assert!(inspect_backup_file(&text_file).is_err());

        // 2. Base de datos SQLite pero sin tabla emotes
        let other_db = tmp_dir.join("other.db");
        let conn = Connection::open(&other_db).unwrap();
        conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY);", [])
            .unwrap();
        drop(conn);

        let err = inspect_backup_file(&other_db).unwrap_err();
        assert!(err.contains("emotes"));

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_record_and_get_app_version() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);",
            [],
        )
        .unwrap();

        assert_eq!(get_recorded_app_version(&conn), None);

        record_app_version(&conn, "0.1.0").unwrap();
        assert_eq!(get_recorded_app_version(&conn), Some("0.1.0".to_string()));

        record_app_version(&conn, "0.1.1").unwrap();
        assert_eq!(get_recorded_app_version(&conn), Some("0.1.1".to_string()));
    }

    #[test]
    fn test_get_and_set_minimize_to_tray() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);",
            [],
        )
        .unwrap();

        // Por defecto es false
        assert!(!get_minimize_to_tray(&conn));

        // Establecer en true
        set_minimize_to_tray(&conn, true).unwrap();
        assert!(get_minimize_to_tray(&conn));

        // Establecer en false
        set_minimize_to_tray(&conn, false).unwrap();
        assert!(!get_minimize_to_tray(&conn));
    }

    #[test]
    fn test_get_and_set_minimize_on_copy() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);",
            [],
        )
        .unwrap();

        // Por defecto es false
        assert!(!get_minimize_on_copy(&conn));

        // Establecer en true
        set_minimize_on_copy(&conn, true).unwrap();
        assert!(get_minimize_on_copy(&conn));

        // Establecer en false
        set_minimize_on_copy(&conn, false).unwrap();
        assert!(!get_minimize_on_copy(&conn));
    }

    #[test]
    fn test_get_and_set_force_gif_loop() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);",
            [],
        )
        .unwrap();

        // Por defecto es true
        assert!(get_force_gif_loop(&conn));

        // Establecer en false
        set_force_gif_loop(&conn, false).unwrap();
        assert!(!get_force_gif_loop(&conn));

        // Establecer en true
        set_force_gif_loop(&conn, true).unwrap();
        assert!(get_force_gif_loop(&conn));
    }

    #[test]
    fn test_get_and_set_global_shortcut() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);",
            [],
        )
        .unwrap();

        // Por defecto es DEFAULT_GLOBAL_SHORTCUT
        assert_eq!(get_global_shortcut(&conn), DEFAULT_GLOBAL_SHORTCUT);

        // Modificar a un atajo personalizado
        set_global_shortcut(&conn, "Alt+Shift+Period").unwrap();
        assert_eq!(get_global_shortcut(&conn), "Alt+Shift+Period");

        // Espacios en blanco alrededor se limpian
        set_global_shortcut(&conn, "  Control+KeyK  ").unwrap();
        assert_eq!(get_global_shortcut(&conn), "Control+KeyK");
    }

    #[test]
    fn test_default_global_shortcut_parses() {
        use tauri_plugin_global_shortcut::Shortcut;
        let parsed: Result<Shortcut, _> = DEFAULT_GLOBAL_SHORTCUT.parse();
        assert!(parsed.is_ok(), "DEFAULT_GLOBAL_SHORTCUT should parse: {:?}", parsed.err());
    }

    #[test]
    fn test_check_and_backup_on_version_upgrade() {
        let tmp_dir = std::env::temp_dir().join(format!("test_upgrade_backup_{}", uuid::Uuid::new_v4()));
        let backups_dir = tmp_dir.join("backups");
        fs::create_dir_all(&backups_dir).unwrap();

        let db_file = tmp_dir.join("test.db");
        let conn = Connection::open(&db_file).unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE emotes (id INTEGER PRIMARY KEY, name TEXT, path TEXT);
             INSERT INTO emotes (id, name, path) VALUES (1, 'doge', 'C:/doge.webp');",
        )
        .unwrap();

        // 1. Simular versión anterior instalada "0.1.0"
        record_app_version(&conn, "0.1.0").unwrap();

        // 2. Se instala nueva versión "0.2.0" por encima
        let backup_result = check_and_backup_version_in_dir(&conn, "0.2.0", &backups_dir).unwrap();

        // Verificamos que se creó la copia de seguridad
        assert!(backup_result.is_some());
        let info = backup_result.unwrap();
        assert_eq!(info.emote_count, 1);
        assert!(info.file_name.starts_with("pre_upgrade_v0.1.0_to_v0.2.0_"));
        assert!(Path::new(&info.path).exists());

        // Verificamos que la versión registrada se actualizó a "0.2.0"
        assert_eq!(get_recorded_app_version(&conn), Some("0.2.0".to_string()));

        // 3. Ejecutar nuevamente con "0.2.0" no debe volver a crear copia
        let rerun_result = check_and_backup_version_in_dir(&conn, "0.2.0", &backups_dir).unwrap();
        assert!(rerun_result.is_none());

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_prune_backups_dir_to_limit() {
        let tmp_dir = std::env::temp_dir().join(format!("test_prune_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).unwrap();

        // Creamos 18 archivos de prueba simulando backups
        for i in 1..=18 {
            let file_path = tmp_dir.join(format!("backup_{:02}.db", i));
            fs::write(&file_path, "sqlite format").unwrap();
            // Espaciar un milisegundo para ordenación determinista por modified time si el SO lo soporta
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        // Podar a un máximo de 15
        let pruned = prune_backups_dir(&tmp_dir, 15).unwrap();
        assert_eq!(pruned, 3);

        let remaining = fs::read_dir(&tmp_dir)
            .unwrap()
            .flatten()
            .filter(|e| e.path().is_file())
            .count();
        assert_eq!(remaining, 15);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_delete_db_backup() {
        let tmp_dir = std::env::temp_dir().join(format!("test_delete_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).unwrap();

        let db_file = tmp_dir.join("sample.db");
        fs::write(&db_file, "content").unwrap();
        assert!(db_file.exists());

        let res = delete_db_backup(&db_file).unwrap();
        assert!(res);
        assert!(!db_file.exists());

        // Archivo que no es .db debe ser rechazado
        let txt_file = tmp_dir.join("sample.txt");
        fs::write(&txt_file, "content").unwrap();
        assert!(delete_db_backup(&txt_file).is_err());
        assert!(txt_file.exists());

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
