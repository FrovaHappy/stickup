mod clipboard;
mod commands;
mod db;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            commands::resync_emotes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
