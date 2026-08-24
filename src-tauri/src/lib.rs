pub mod commands;
pub mod error;
pub mod hotkey;
pub mod llm;
pub mod models;
pub mod pipeline;
pub mod search;
pub mod storage;
pub mod tray;

use commands::analysis::analyze_text;
use commands::history::{clear_history, delete_analysis, get_analysis, list_history};
use commands::settings::{clear_api_key, get_settings, has_api_key, set_api_key, set_settings};
use commands::updates::check_latest_release;
pub use error::{AppError, AppResult};
use storage::db::Db;
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
            tray::focus_main(app);
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            set_api_key,
            clear_api_key,
            has_api_key,
            analyze_text,
            check_latest_release,
            list_history,
            get_analysis,
            delete_analysis,
            clear_history,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                    tray::set_dock_icon_visible(window.app_handle(), false);
                }
            }
        })
        .setup(|app| {
            let store = app.store(SETTINGS_FILE)?;
            let settings: Settings = store
                .get(SETTINGS_KEY)
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_else(Settings::with_system_locale);

            let data_dir = app.path().app_data_dir()?;
            let db = Db::open(data_dir.join("cache.db"))?;

            // Enforce the retention window once per launch, before anything can
            // read history — deleting is the user's privacy expectation, not a
            // background nicety, so a failure is logged rather than swallowed.
            if let Some(days) = settings.history_retention_days {
                let cutoff =
                    chrono::Utc::now().timestamp_millis() - i64::from(days) * 24 * 3_600 * 1_000;
                match storage::history::prune(&db, cutoff) {
                    Ok(0) => {}
                    Ok(removed) => tracing::info!("pruned {removed} history entries"),
                    Err(error) => tracing::warn!("history prune failed: {error}"),
                }
            }

            app.manage(db);
            hotkey::install(&app.handle().clone(), &settings.hotkey)?;
            tray::install(&app.handle().clone(), &settings.locale)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
