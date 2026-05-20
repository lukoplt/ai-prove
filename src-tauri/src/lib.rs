pub mod commands;
pub mod error;
pub mod hotkey;
pub mod models;
pub mod storage;
pub mod tray;

use commands::settings::{clear_api_key, get_settings, has_api_key, set_api_key, set_settings};
pub use error::{AppError, AppResult};
use storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use tauri::{Manager, WindowEvent};
use tauri_plugin_store::StoreExt;
use tracing_subscriber::{fmt, EnvFilter};

pub fn run() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            set_api_key,
            clear_api_key,
            has_api_key,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let store = app.store(SETTINGS_FILE)?;
            let settings: Settings = store
                .get(SETTINGS_KEY)
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default();

            hotkey::install(&app.handle().clone(), &settings.hotkey)?;
            tray::install(&app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
