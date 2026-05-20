use crate::error::{AppError, AppResult};
use crate::storage::keychain;
use crate::storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use serde_json::json;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub async fn get_settings<R: Runtime>(app: AppHandle<R>) -> AppResult<Settings> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AppError::Store(error.to_string()))?;

    let value = store.get(SETTINGS_KEY);
    let settings: Settings = match value {
        Some(value) => serde_json::from_value(value).unwrap_or_else(|_| Settings::with_system_locale()),
        None => Settings::with_system_locale(),
    };

    Ok(settings)
}

#[tauri::command]
pub async fn set_settings<R: Runtime>(app: AppHandle<R>, settings: Settings) -> AppResult<()> {
    settings.validate()?;

    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AppError::Store(error.to_string()))?;
    store.set(SETTINGS_KEY, json!(settings));
    store
        .save()
        .map_err(|error| AppError::Store(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(account: String, secret: String) -> AppResult<()> {
    if secret.trim().is_empty() {
        return Err(AppError::Invalid("api key is empty".into()));
    }

    keychain::set_api_key(&account, secret.trim())
}

#[tauri::command]
pub async fn clear_api_key(account: String) -> AppResult<()> {
    keychain::clear_api_key(&account)
}

#[tauri::command]
pub async fn has_api_key(account: String) -> AppResult<bool> {
    Ok(keychain::get_api_key(&account)?.is_some())
}
