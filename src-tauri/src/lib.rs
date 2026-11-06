mod clipboard;
mod commands;
mod db;
mod models;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Asegura la base de datos y crea la copia automática por actualización antes de iniciar
            let _ = db::ensure_db();

            // Asegura que la ventana y barra de tareas carguen el icono activo en desarrollo
            if let Some(icon) = app.default_window_icon() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon.clone());
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_emotes,
            commands::import_paths,
            commands::import_raw_bytes,
            commands::import_from_url,
            commands::import_from_clipboard,
            commands::get_storage_path,
            commands::set_storage_path,
            commands::delete_emote,
            commands::update_tag,
            commands::copy_to_clipboard,
            commands::record_interaction,
            commands::resync_emotes,
            commands::create_backup,
            commands::restore_backup,
            commands::delete_backup,
            commands::list_backups,
            commands::inspect_backup,
            commands::get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
