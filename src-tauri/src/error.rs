use serde::ser::SerializeStruct;
use serde::Serialize;
use thiserror::Error;

/// Stable, machine-readable classification of a failure. The frontend maps
/// each code to a localized sentence plus a remedy; the human-readable
/// `message` is kept only as collapsible diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The configured CLI program is not on PATH / not executable.
    CliNotFound,
    /// The CLI ran but exited non-zero.
    CliFailed,
    /// The CLI exceeded `CLI_TIMEOUT`.
    CliTimeout,
    /// The CLI produced output we could not read as a JSON object.
    CliBadOutput,
    /// The LLM API rejected our credentials (401/403).
    LlmAuth,
    /// The LLM API rate-limited us (429).
    LlmRateLimit,
    /// Any other non-success status from the LLM API.
    LlmHttp,
    /// The search API rejected our credentials (401/403).
    SearchAuth,
    /// The search API rate-limited us (429).
    SearchRateLimit,
    /// Any other non-success status from the search API.
    SearchHttp,
    /// Transport-level failure: DNS, TLS, connection refused, timeout.
    Network,
    Keychain,
    Store,
    Io,
    Serde,
    Tauri,
    Hotkey,
    NotFound,
    Invalid,
    Other,
}

/// Maps an HTTP status onto the auth / rate-limit / generic triple a caller
/// supplies, so the LLM and search clients classify statuses identically.
#[must_use]
pub fn http_error_code(
    status: u16,
    auth: ErrorCode,
    rate_limit: ErrorCode,
    other: ErrorCode,
) -> ErrorCode {
    match status {
        401 | 403 => auth,
        429 => rate_limit,
        _ => other,
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("keychain error: {0}")]
    Keychain(#[from] keyring::Error),

    #[error("store error: {0}")]
    Store(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("global shortcut error: {0}")]
    GlobalShortcut(#[from] tauri_plugin_global_shortcut::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    Invalid(String),

    /// A classified provider failure. `detail` is raw diagnostics — it may
    /// contain a CLI's stderr and is shown only behind a disclosure control.
    #[error("{detail}")]
    Provider { code: ErrorCode, detail: String },

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    #[must_use]
    pub fn provider(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self::Provider {
            code,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Provider { code, .. } => *code,
            Self::Keychain(_) => ErrorCode::Keychain,
            Self::Store(_) => ErrorCode::Store,
            Self::Io(_) => ErrorCode::Io,
            Self::Serde(_) => ErrorCode::Serde,
            Self::Tauri(_) => ErrorCode::Tauri,
            Self::GlobalShortcut(_) => ErrorCode::Hotkey,
            Self::NotFound(_) => ErrorCode::NotFound,
            Self::Invalid(_) => ErrorCode::Invalid,
            Self::Other(_) => ErrorCode::Other,
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", &self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_error_reports_its_code() {
        let error = AppError::provider(ErrorCode::CliNotFound, "claude not on PATH");
        assert_eq!(error.code(), ErrorCode::CliNotFound);
        assert_eq!(error.to_string(), "claude not on PATH");
    }

    #[test]
    fn plain_variants_map_to_stable_codes() {
        assert_eq!(AppError::Invalid("x".into()).code(), ErrorCode::Invalid);
        assert_eq!(AppError::NotFound("x".into()).code(), ErrorCode::NotFound);
        assert_eq!(AppError::Store("x".into()).code(), ErrorCode::Store);
        assert_eq!(AppError::Other("x".into()).code(), ErrorCode::Other);
    }

    #[test]
    fn serializes_as_code_and_message() {
        let error = AppError::provider(ErrorCode::LlmRateLimit, "429 slow down");
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "llm_rate_limit");
        assert_eq!(json["message"], "429 slow down");
    }

    #[test]
    fn every_code_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(ErrorCode::CliBadOutput).unwrap(),
            "cli_bad_output"
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::SearchRateLimit).unwrap(),
            "search_rate_limit"
        );
    }

    #[test]
    fn http_status_maps_to_auth_rate_limit_or_generic() {
        assert_eq!(
            http_error_code(
                401,
                ErrorCode::LlmAuth,
                ErrorCode::LlmRateLimit,
                ErrorCode::LlmHttp
            ),
            ErrorCode::LlmAuth
        );
        assert_eq!(
            http_error_code(
                429,
                ErrorCode::SearchAuth,
                ErrorCode::SearchRateLimit,
                ErrorCode::SearchHttp
            ),
            ErrorCode::SearchRateLimit
        );
        assert_eq!(
            http_error_code(
                500,
                ErrorCode::LlmAuth,
                ErrorCode::LlmRateLimit,
                ErrorCode::LlmHttp
            ),
            ErrorCode::LlmHttp
        );
    }
}
