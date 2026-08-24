use crate::error::{AppError, AppResult};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::{info, warn};

pub const DEFAULT_ACCELERATOR: &str = "CommandOrControl+Shift+D";

pub fn install<R: Runtime>(app: &AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let parsed: Shortcut = match accelerator.parse() {
        Ok(shortcut) => shortcut,
        Err(error) => {
            warn!("hotkey {accelerator} invalid ({error}); falling back to default");
            DEFAULT_ACCELERATOR
                .parse()
                .expect("default accelerator parses")
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

/// Validates an accelerator string by parsing it the same way registration
/// will. Returns the trimmed form so callers persist a canonical value.
pub fn normalize(accelerator: &str) -> AppResult<String> {
    let trimmed = accelerator.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("hotkey cannot be empty".into()));
    }

    trimmed
        .parse::<Shortcut>()
        .map_err(|error| AppError::Invalid(format!("invalid hotkey '{trimmed}': {error}")))?;

    Ok(trimmed.to_string())
}

/// Drops every registered shortcut and registers `accelerator`. Used when the
/// user remaps the hotkey at runtime, so the change takes effect immediately
/// instead of at the next launch.
pub fn reinstall<R: Runtime>(app: &AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let accelerator = normalize(accelerator)?;
    app.global_shortcut().unregister_all()?;
    install(app, &accelerator)
}

fn handle_trigger<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
        let _ = app.emit("capture-trigger", ());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_accepts_the_default_accelerator() {
        assert_eq!(normalize(DEFAULT_ACCELERATOR).unwrap(), DEFAULT_ACCELERATOR);
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(normalize("  Alt+F5  ").unwrap(), "Alt+F5");
    }

    #[test]
    fn normalize_rejects_empty_input() {
        assert!(normalize("   ").is_err());
    }

    #[test]
    fn normalize_rejects_garbage() {
        assert!(normalize("NotAKey+++").is_err());
    }
}
