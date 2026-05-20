use crate::error::AppResult;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::{info, warn};

pub fn install<R: Runtime>(app: &AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let parsed: Shortcut = match accelerator.parse() {
        Ok(shortcut) => shortcut,
        Err(error) => {
            warn!("hotkey {accelerator} invalid ({error}); falling back to default");
            "CommandOrControl+Shift+D".parse().unwrap()
        }
    };

    let app_for_handler = app.clone();
    app.global_shortcut()
        .on_shortcut(parsed, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                handle_trigger(&app_for_handler);
            }
        })?;

    info!("hotkey installed: {accelerator}");
    Ok(())
}

fn handle_trigger<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
        let _ = app.emit("capture-trigger", ());
    }
}
