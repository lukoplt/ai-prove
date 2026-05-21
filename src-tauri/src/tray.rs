use crate::error::AppResult;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

pub fn install<R: Runtime>(app: &AppHandle<R>, locale: &str) -> AppResult<TrayIcon<R>> {
    let (show_label, quit_label) = match locale {
        "cs" => ("Otevřít Druhý názor", "Ukončit"),
        _ => ("Open Druhý názor", "Quit"),
    };
    let show_item = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => focus_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if matches!(button, tauri::tray::MouseButton::Left) {
                    focus_main(tray.app_handle());
                }
            }
        })
        .build(app)?;
    Ok(tray)
}

fn focus_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
