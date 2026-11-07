mod clipboard;
mod commands;
mod db;
mod models;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub struct TrayState {
    pub minimize_to_tray: AtomicBool,
}

#[cfg(not(test))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(|app| {
            // Asegura la base de datos y crea la copia automática por actualización antes de iniciar
            let initial_minimize = if let Ok(conn) = db::ensure_db() {
                db::get_minimize_to_tray(&conn)
            } else {
                false
            };

            app.manage(TrayState {
                minimize_to_tray: AtomicBool::new(initial_minimize),
            });

            // Asegura que la ventana y barra de tareas carguen el icono activo en desarrollo
            if let Some(icon) = app.default_window_icon() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon.clone());
                }
            }

            // Si se inició con el argumento silencioso (--hidden), mantener oculta la ventana principal
            let is_hidden_launch =
                std::env::args().any(|arg| arg == "--hidden" || arg == "--minimized");
            if is_hidden_launch {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            // Configuración del icono en la bandeja del sistema (System Tray)
            let show_item = MenuItem::with_id(app, "show", "Mostrar StickUp", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::with_id("stickup-tray")
                .tooltip("StickUp - Gestor de Emotes")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _ = tray_builder.build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<TrayState>();
                if state.minimize_to_tray.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
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
            commands::get_app_version,
            commands::get_minimize_to_tray,
            commands::set_minimize_to_tray
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
