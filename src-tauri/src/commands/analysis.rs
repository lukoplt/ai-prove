use crate::error::{AppError, AppResult};
use crate::llm::anthropic::AnthropicProvider;
use crate::llm::cli::CliProvider;
use crate::llm::LlmProvider;
use crate::models::{Analysis, Claim, ClaimKind, Verification, VerificationStatus};
use crate::pipeline::atomize_to_claims;
use crate::pipeline::verify::VerificationEngine;
use crate::search::brave::BraveClient;
use crate::search::extract::Extractor;
use crate::search::SearchProvider;
use crate::storage::cache;
use crate::storage::db::Db;
use crate::storage::keychain;
use crate::storage::settings_store::{ProviderKind, Settings, SETTINGS_FILE, SETTINGS_KEY};
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Runtime, State};
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

pub const ACCOUNT_ANTHROPIC: &str = "anthropic";
pub const ACCOUNT_BRAVE: &str = "brave";
pub const MAX_VERIFIED_CLAIMS: usize = 8;

fn build_llm_provider(settings: &Settings) -> AppResult<Arc<dyn LlmProvider>> {
    match settings.provider {
        ProviderKind::Anthropic => {
            let key = keychain::get_api_key(ACCOUNT_ANTHROPIC)?
                .ok_or_else(|| AppError::NotFound("anthropic key".into()))?;
            let provider = AnthropicProvider::new(
                key,
                settings.anthropic_model.clone(),
                settings.locale.clone(),
            )?;
            Ok(Arc::new(provider))
        }
        ProviderKind::Cli => {
            let provider = CliProvider::new(&settings.cli_command, settings.locale.clone())?;
            Ok(Arc::new(provider))
        }
    }
}

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

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClaimVerifiedEvent {
    pub analysis_id: String,
    pub claim_id: String,
    pub verification: Verification,
}

#[tauri::command]
pub async fn analyze_text<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    text: String,
) -> AppResult<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("input is empty".into()));
    }

    let settings = load_settings(&app);
    let brave_key = keychain::get_api_key(ACCOUNT_BRAVE)?.filter(|key| !key.trim().is_empty());
    let provider = build_llm_provider(&settings)?;

    let analysis_id = Uuid::now_v7().to_string();
    app.emit(
        "analysis-started",
        AnalysisStartedEvent {
            analysis_id: analysis_id.clone(),
        },
    )
    .map_err(|error| AppError::Other(format!("emit: {error}")))?;

    let outcome = atomize_to_claims(provider.as_ref(), trimmed).await?;
    let analysis = Analysis {
        id: analysis_id.clone(),
        created_at: Utc::now().timestamp_millis(),
        input: trimmed.to_string(),
        claims: outcome.claims.clone(),
        truncated: outcome.truncated,
    };

    app.emit(
        "analysis-claims",
        AnalysisClaimsEvent {
            analysis_id: analysis_id.clone(),
            analysis: analysis.clone(),
        },
    )
    .map_err(|error| AppError::Other(format!("emit: {error}")))?;

    spawn_verifications(
        &app,
        db.inner().clone(),
        provider,
        brave_key,
        analysis_id.clone(),
        trimmed.to_string(),
        analysis.created_at,
        outcome.claims,
        analysis.truncated,
        settings.cache_ttl_days,
        &settings.locale,
    );

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

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn spawn_verifications<R: Runtime>(
    app: &AppHandle<R>,
    db: Db,
    provider: Arc<dyn LlmProvider>,
    brave_key: Option<String>,
    analysis_id: String,
    input: String,
    created_at_ms: i64,
    claims: Vec<Claim>,
    truncated: bool,
    cache_ttl_days: u32,
    locale: &str,
) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::{Mutex, Notify};
    use tokio::time::{timeout, Duration};

    let baseline = Analysis {
        id: analysis_id.clone(),
        created_at: created_at_ms,
        input: input.clone(),
        claims: claims.clone(),
        truncated,
    };
    let _ = crate::storage::history::insert(&db, &baseline);

    let all_fact_claims = collect_fact_claims(&claims);
    let (fact_claims, skipped_fact_claims) = split_fact_claims(&claims);

    if all_fact_claims.is_empty() {
        return;
    }

    let mut final_claims_vec = claims;

    let Some(brave_key) = brave_key else {
        mark_web_search_disabled_fact_claims(
            app,
            &analysis_id,
            &mut final_claims_vec,
            all_fact_claims,
            locale,
        );
        let analysis = Analysis {
            id: analysis_id,
            created_at: created_at_ms,
            input,
            claims: final_claims_vec,
            truncated,
        };
        let _ = crate::storage::history::insert(&db, &analysis);
        return;
    };

    mark_skipped_fact_claims(
        app,
        &analysis_id,
        &mut final_claims_vec,
        skipped_fact_claims,
        locale,
    );

    let extractor = match Extractor::new() {
        Ok(extractor) => Arc::new(extractor),
        Err(_) => return,
    };
    let search: Arc<dyn SearchProvider> = match BraveClient::new(brave_key, locale.to_string()) {
        Ok(client) => Arc::new(client),
        Err(_) => return,
    };
    let engine = Arc::new(VerificationEngine {
        llm: provider,
        search,
        extractor,
        locale: locale.to_string(),
    });

    let app = app.clone();
    let ttl_ms = i64::from(cache_ttl_days) * 24 * 3_600 * 1000;
    let total = fact_claims.len();
    let done = Arc::new(AtomicUsize::new(0));
    let notify = Arc::new(Notify::new());
    let final_claims = Arc::new(Mutex::new(final_claims_vec));

    for claim in fact_claims {
        let app = app.clone();
        let db = db.clone();
        let engine = engine.clone();
        let analysis_id = analysis_id.clone();
        let done = done.clone();
        let notify = notify.clone();
        let final_claims = final_claims.clone();
        let locale = locale.to_string();

        tokio::spawn(async move {
            let now = Utc::now().timestamp_millis();
            let hash = cache::hash_claim(&claim.text);
            let verification = match cache::get(&db, &hash, ttl_ms, now) {
                Ok(Some(cached)) => cached,
                _ => match engine.verify(&claim.text).await {
                    Ok(verification) => {
                        let _ = cache::put(&db, &hash, &claim.text, &verification, now);
                        verification
                    }
                    Err(error) => Verification {
                        status: VerificationStatus::NotFound,
                        sources: Vec::new(),
                        summary: verification_failure_message(&locale, &error.to_string()),
                    },
                },
            };

            {
                let mut claims = final_claims.lock().await;
                if let Some(target) = claims.iter_mut().find(|item| item.id == claim.id) {
                    target.verification = Some(verification.clone());
                }
            }

            let _ = app.emit(
                "claim-verified",
                ClaimVerifiedEvent {
                    analysis_id,
                    claim_id: claim.id,
                    verification,
                },
            );

            let previous = done.fetch_add(1, Ordering::SeqCst);
            if previous + 1 >= total {
                notify.notify_one();
            }
        });
    }

    tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(30), notify.notified()).await;
        let claims_snapshot = final_claims.lock().await.clone();
        let analysis = Analysis {
            id: analysis_id,
            created_at: created_at_ms,
            input,
            claims: claims_snapshot,
            truncated,
        };
        let _ = crate::storage::history::insert(&db, &analysis);
    });
}

fn collect_fact_claims(claims: &[Claim]) -> Vec<Claim> {
    claims
        .iter()
        .filter(|claim| claim.kind == ClaimKind::Fact)
        .cloned()
        .collect()
}

fn split_fact_claims(claims: &[Claim]) -> (Vec<Claim>, Vec<Claim>) {
    let all_fact_claims = collect_fact_claims(claims);
    let verified = all_fact_claims
        .iter()
        .take(MAX_VERIFIED_CLAIMS)
        .cloned()
        .collect();
    let skipped = all_fact_claims
        .into_iter()
        .skip(MAX_VERIFIED_CLAIMS)
        .collect();

    (verified, skipped)
}

fn mark_skipped_fact_claims<R: Runtime>(
    app: &AppHandle<R>,
    analysis_id: &str,
    claims: &mut [Claim],
    skipped_fact_claims: Vec<Claim>,
    locale: &str,
) {
    for claim in skipped_fact_claims {
        let verification = skipped_fact_verification(locale);
        if let Some(target) = claims.iter_mut().find(|candidate| candidate.id == claim.id) {
            target.verification = Some(verification.clone());
        }

        let _ = app.emit(
            "claim-verified",
            ClaimVerifiedEvent {
                analysis_id: analysis_id.to_string(),
                claim_id: claim.id,
                verification,
            },
        );
    }
}

fn mark_web_search_disabled_fact_claims<R: Runtime>(
    app: &AppHandle<R>,
    analysis_id: &str,
    claims: &mut [Claim],
    fact_claims: Vec<Claim>,
    locale: &str,
) {
    for claim in fact_claims {
        let verification = web_search_disabled_verification(locale);
        if let Some(target) = claims.iter_mut().find(|candidate| candidate.id == claim.id) {
            target.verification = Some(verification.clone());
        }

        let _ = app.emit(
            "claim-verified",
            ClaimVerifiedEvent {
                analysis_id: analysis_id.to_string(),
                claim_id: claim.id,
                verification,
            },
        );
    }
}

#[must_use]
fn skipped_fact_verification(locale: &str) -> Verification {
    let summary = match locale {
        "cs" => format!("Ověřuje se jen prvních {MAX_VERIFIED_CLAIMS} faktických tvrzení."),
        _ => format!("Only the first {MAX_VERIFIED_CLAIMS} factual claims are verified."),
    };
    Verification {
        status: VerificationStatus::NotVerified,
        sources: Vec::new(),
        summary,
    }
}

#[must_use]
fn web_search_disabled_verification(locale: &str) -> Verification {
    let summary = match locale {
        "cs" => "Webové ověřování není zapnuté. Tvrzení bylo zpracováno bez hledání zdrojů.".into(),
        _ => {
            "Web verification is not enabled. The claim was processed without source search.".into()
        }
    };
    Verification {
        status: VerificationStatus::NotVerified,
        sources: Vec::new(),
        summary,
    }
}

fn verification_failure_message(locale: &str, error: &str) -> String {
    match locale {
        "cs" => format!("Verifikace selhala: {error}"),
        _ => format!("Verification failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_search_disabled_verification_is_not_verified_without_sources() {
        let verification = web_search_disabled_verification("cs");

        assert_eq!(verification.status, VerificationStatus::NotVerified);
        assert!(verification.sources.is_empty());
        assert_eq!(
            verification.summary,
            "Webové ověřování není zapnuté. Tvrzení bylo zpracováno bez hledání zdrojů."
        );
    }
}
