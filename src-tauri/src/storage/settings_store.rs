use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE: &str = "settings.json";
pub const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub locale: String,
    pub hotkey: String,
    pub model: String,
    pub cache_ttl_days: u32,
    pub onboarded: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: "en".to_string(),
            hotkey: "CommandOrControl+Shift+D".to_string(),
            model: "claude-haiku-4-5-20251001".to_string(),
            cache_ttl_days: 7,
            onboarded: false,
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
    fn settings_roundtrip_json() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, back);
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
