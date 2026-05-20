use crate::error::{AppError, AppResult};
use crate::llm::anthropic::AnthropicProvider;
use crate::models::Analysis;
use crate::pipeline::atomize_to_claims;
use crate::storage::keychain;
use crate::storage::settings_store::{Settings, SETTINGS_FILE, SETTINGS_KEY};
use chrono::Utc;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

pub const ACCOUNT_ANTHROPIC: &str = "anthropic";

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisStartedEvent {
    pub analysis_id: String,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisClaimsEvent {
    pub analysis_id: String,
    pub analysis: Analysis,
}

#[tauri::command]
pub async fn analyze_text<R: Runtime>(app: AppHandle<R>, text: String) -> AppResult<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("input is empty".into()));
    }

    let api_key = keychain::get_api_key(ACCOUNT_ANTHROPIC)?
        .ok_or_else(|| AppError::NotFound("anthropic key".into()))?;
    let settings = load_settings(&app);
    let provider = AnthropicProvider::new(api_key, settings.model, settings.locale)?;

    let analysis_id = Uuid::now_v7().to_string();
    app.emit(
        "analysis-started",
        AnalysisStartedEvent {
            analysis_id: analysis_id.clone(),
        },
    )
    .map_err(|error| AppError::Other(format!("emit: {error}")))?;

    let outcome = atomize_to_claims(&provider, trimmed).await?;
    let analysis = Analysis {
        id: analysis_id.clone(),
        created_at: Utc::now().timestamp_millis(),
        input: trimmed.to_string(),
        claims: outcome.claims,
        truncated: outcome.truncated,
    };

    app.emit(
        "analysis-claims",
        AnalysisClaimsEvent {
            analysis_id: analysis_id.clone(),
            analysis,
        },
    )
    .map_err(|error| AppError::Other(format!("emit: {error}")))?;

    Ok(analysis_id)
}

fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Settings {
    let Some(settings) = app
        .store(SETTINGS_FILE)
        .ok()
        .and_then(|store| store.get(SETTINGS_KEY))
        .and_then(|value| serde_json::from_value(value).ok())
    else {
        return Settings::with_system_locale();
    };

    settings
}
