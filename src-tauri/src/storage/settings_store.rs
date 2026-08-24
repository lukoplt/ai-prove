use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE: &str = "settings.json";
pub const SETTINGS_KEY: &str = "settings";

pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-haiku-4-5-20251001";
pub const DEFAULT_CLI_COMMAND: &str = "claude -p";

/// Default number of factual claims verified against the web per analysis.
pub const DEFAULT_VERIFIED_CLAIMS_LIMIT: u32 = 8;

/// Default history retention. `None` in `Settings` means "keep forever".
pub const DEFAULT_HISTORY_RETENTION_DAYS: u32 = 90;
pub const MAX_HISTORY_RETENTION_DAYS: u32 = 3650;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    #[default]
    Cli,
    Anthropic,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePref {
    #[default]
    Auto,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub locale: String,
    pub hotkey: String,
    pub cache_ttl_days: u32,
    pub onboarded: bool,

    #[serde(default)]
    pub provider: ProviderKind,

    /// Anthropic model id (when `provider == Anthropic`).
    #[serde(default = "default_anthropic_model")]
    pub anthropic_model: String,

    /// Shell command for the CLI provider (when `provider == Cli`).
    /// Parsed via `shlex::split`. Example: `"claude -p"`, `"ollama run llama3.2 --format json"`.
    #[serde(default = "default_cli_command")]
    pub cli_command: String,

    /// When true, the app makes one GET request to the GitHub Releases API on
    /// launch to discover newer published versions. No data is sent; the
    /// fetched manifest is read locally. Default off.
    #[serde(default)]
    pub check_updates_on_launch: bool,

    /// UI theme preference. `Auto` follows the OS color scheme.
    #[serde(default)]
    pub theme: ThemePref,

    /// Forces the high-contrast palette regardless of the OS setting: opaque
    /// surfaces, no blur, no mesh background.
    #[serde(default)]
    pub high_contrast: bool,

    /// When true, the app asks for explicit confirmation before the first byte
    /// of the user's text leaves the process. Default on — the disclosure is
    /// the point of the app's privacy promise, so opting out must be deliberate.
    #[serde(default = "default_true")]
    pub confirm_before_send: bool,

    /// How long analyses stay in the local history, in days. `None` keeps them
    /// forever. Pruned once at startup.
    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: Option<u32>,

    /// How many factual claims are verified against the web per analysis.
    /// `None` means "all" — every factual claim is verified (up to the
    /// atomization cap of `MAX_CLAIMS`). `Some(n)` verifies only the first `n`
    /// factual claims; the rest are marked `NotVerified`. Default `Some(8)`.
    #[serde(default = "default_verified_claims_limit")]
    pub verified_claims_limit: Option<u32>,
}

const fn default_true() -> bool {
    true
}

fn default_anthropic_model() -> String {
    DEFAULT_ANTHROPIC_MODEL.to_string()
}

fn default_cli_command() -> String {
    DEFAULT_CLI_COMMAND.to_string()
}

#[allow(clippy::unnecessary_wraps)] // serde default for an `Option<u32>` field
fn default_history_retention_days() -> Option<u32> {
    Some(DEFAULT_HISTORY_RETENTION_DAYS)
}

#[allow(clippy::unnecessary_wraps)] // serde default for an `Option<u32>` field
fn default_verified_claims_limit() -> Option<u32> {
    Some(DEFAULT_VERIFIED_CLAIMS_LIMIT)
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: "en".to_string(),
            hotkey: "CommandOrControl+Shift+D".to_string(),
            cache_ttl_days: 7,
            onboarded: false,
            provider: ProviderKind::Cli,
            anthropic_model: DEFAULT_ANTHROPIC_MODEL.to_string(),
            cli_command: DEFAULT_CLI_COMMAND.to_string(),
            check_updates_on_launch: false,
            theme: ThemePref::Auto,
            high_contrast: false,
            confirm_before_send: true,
            history_retention_days: Some(DEFAULT_HISTORY_RETENTION_DAYS),
            verified_claims_limit: Some(DEFAULT_VERIFIED_CLAIMS_LIMIT),
        }
    }
}

impl Settings {
    #[must_use]
    pub fn map_locale(raw: &str) -> String {
        let two = raw
            .split(['-', '_'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match two.as_str() {
            "cs" => "cs".into(),
            _ => "en".into(),
        }
    }

    #[must_use]
    pub fn with_system_locale() -> Self {
        let detected = sys_locale::get_locale()
            .as_deref()
            .map_or_else(|| "en".into(), Self::map_locale);
        Self {
            locale: detected,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.locale != "cs" && self.locale != "en" {
            return Err(AppError::Invalid(format!(
                "locale must be cs or en, got {}",
                self.locale
            )));
        }

        if self.cache_ttl_days == 0 || self.cache_ttl_days > 90 {
            return Err(AppError::Invalid(format!(
                "cache_ttl_days out of range (1..=90), got {}",
                self.cache_ttl_days
            )));
        }

        if self.hotkey.trim().is_empty() {
            return Err(AppError::Invalid("hotkey cannot be empty".into()));
        }

        if let Some(days) = self.history_retention_days {
            if days == 0 || days > MAX_HISTORY_RETENTION_DAYS {
                return Err(AppError::Invalid(format!(
                    "history_retention_days out of range (1..={MAX_HISTORY_RETENTION_DAYS} or null to keep forever), got {days}"
                )));
            }
        }

        if let Some(limit) = self.verified_claims_limit {
            let max = crate::pipeline::atomize::MAX_CLAIMS;
            if limit == 0 || limit as usize > max {
                return Err(AppError::Invalid(format!(
                    "verified_claims_limit out of range (1..={max} or null for all), got {limit}"
                )));
            }
        }

        match self.provider {
            ProviderKind::Anthropic => {
                if self.anthropic_model.trim().is_empty() {
                    return Err(AppError::Invalid("anthropic_model cannot be empty".into()));
                }
            }
            ProviderKind::Cli => {
                let parts = shlex::split(&self.cli_command).unwrap_or_default();
                if parts.is_empty() {
                    return Err(AppError::Invalid("cli_command cannot be empty".into()));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_validate() {
        assert!(Settings::default().validate().is_ok());
    }

    #[test]
    fn default_provider_is_cli() {
        assert_eq!(Settings::default().provider, ProviderKind::Cli);
    }

    #[test]
    fn invalid_locale_rejected() {
        let settings = Settings {
            locale: "de".into(),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn zero_ttl_rejected() {
        let settings = Settings {
            cache_ttl_days: 0,
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn ttl_over_max_rejected() {
        let settings = Settings {
            cache_ttl_days: 91,
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn empty_hotkey_rejected() {
        let settings = Settings {
            hotkey: "  ".into(),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn empty_cli_command_rejected() {
        let settings = Settings {
            provider: ProviderKind::Cli,
            cli_command: "   ".into(),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn empty_anthropic_model_rejected() {
        let settings = Settings {
            provider: ProviderKind::Anthropic,
            anthropic_model: "  ".into(),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn cli_command_with_args_validates() {
        let settings = Settings {
            provider: ProviderKind::Cli,
            cli_command: "ollama run llama3.2 --format json".into(),
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn settings_roundtrip_json() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, back);
    }

    #[test]
    fn legacy_settings_without_provider_fields_deserializes_via_defaults() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.provider, ProviderKind::Cli);
        assert_eq!(parsed.cli_command, DEFAULT_CLI_COMMAND);
        assert_eq!(parsed.anthropic_model, DEFAULT_ANTHROPIC_MODEL);
    }

    #[test]
    fn default_theme_is_auto() {
        assert_eq!(Settings::default().theme, ThemePref::Auto);
    }

    #[test]
    fn legacy_settings_without_theme_deserializes_to_auto() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.theme, ThemePref::Auto);
    }

    #[test]
    fn theme_roundtrips_json() {
        let settings = Settings {
            theme: ThemePref::Dark,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, ThemePref::Dark);
    }

    #[test]
    fn default_verified_claims_limit_is_eight() {
        assert_eq!(
            Settings::default().verified_claims_limit,
            Some(DEFAULT_VERIFIED_CLAIMS_LIMIT)
        );
    }

    #[test]
    fn legacy_settings_without_verified_limit_deserializes_to_default() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(
            parsed.verified_claims_limit,
            Some(DEFAULT_VERIFIED_CLAIMS_LIMIT)
        );
    }

    #[test]
    fn verified_limit_all_validates() {
        let settings = Settings {
            verified_claims_limit: None,
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn verified_limit_zero_rejected() {
        let settings = Settings {
            verified_claims_limit: Some(0),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn verified_limit_over_max_rejected() {
        let over_max = u32::try_from(crate::pipeline::atomize::MAX_CLAIMS).unwrap() + 1;
        let settings = Settings {
            verified_claims_limit: Some(over_max),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn verified_limit_roundtrips_json() {
        for limit in [None, Some(1), Some(8), Some(25)] {
            let settings = Settings {
                verified_claims_limit: limit,
                ..Settings::default()
            };
            let json = serde_json::to_string(&settings).unwrap();
            let back: Settings = serde_json::from_str(&json).unwrap();
            assert_eq!(back.verified_claims_limit, limit);
        }
    }

    #[test]
    fn confirm_before_send_defaults_to_true() {
        assert!(Settings::default().confirm_before_send);
    }

    #[test]
    fn legacy_settings_without_confirm_flag_default_to_true() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert!(parsed.confirm_before_send);
    }

    #[test]
    fn confirm_before_send_roundtrips_json() {
        let settings = Settings {
            confirm_before_send: false,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert!(!back.confirm_before_send);
    }

    #[test]
    fn history_retention_defaults_to_ninety_days() {
        assert_eq!(
            Settings::default().history_retention_days,
            Some(DEFAULT_HISTORY_RETENTION_DAYS)
        );
    }

    #[test]
    fn legacy_settings_without_retention_default_to_ninety_days() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert_eq!(
            parsed.history_retention_days,
            Some(DEFAULT_HISTORY_RETENTION_DAYS)
        );
    }

    #[test]
    fn history_retention_forever_validates() {
        let settings = Settings {
            history_retention_days: None,
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn history_retention_zero_rejected() {
        let settings = Settings {
            history_retention_days: Some(0),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn history_retention_over_ten_years_rejected() {
        let settings = Settings {
            history_retention_days: Some(MAX_HISTORY_RETENTION_DAYS + 1),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn high_contrast_defaults_to_false() {
        assert!(!Settings::default().high_contrast);
    }

    #[test]
    fn legacy_settings_without_high_contrast_default_to_false() {
        let legacy = r#"{
            "locale": "cs",
            "hotkey": "CommandOrControl+Shift+D",
            "cache_ttl_days": 7,
            "onboarded": false
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).unwrap();
        assert!(!parsed.high_contrast);
    }

    #[test]
    fn map_locale_cs_variants() {
        assert_eq!(Settings::map_locale("cs-CZ"), "cs");
        assert_eq!(Settings::map_locale("cs_CZ"), "cs");
        assert_eq!(Settings::map_locale("CS"), "cs");
    }

    #[test]
    fn map_locale_en_variants() {
        assert_eq!(Settings::map_locale("en-US"), "en");
        assert_eq!(Settings::map_locale("en_GB"), "en");
        assert_eq!(Settings::map_locale("EN"), "en");
    }

    #[test]
    fn map_locale_unsupported_falls_back_to_en() {
        assert_eq!(Settings::map_locale("de-DE"), "en");
        assert_eq!(Settings::map_locale("fr"), "en");
        assert_eq!(Settings::map_locale(""), "en");
    }

    #[test]
    fn with_system_locale_produces_supported_locale() {
        let settings = Settings::with_system_locale();
        assert!(settings.locale == "cs" || settings.locale == "en");
    }
}
