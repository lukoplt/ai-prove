use crate::storage::settings_store::ProviderKind;

const ATOMIZE_CS: &str = include_str!("atomize_cs.txt");
const ATOMIZE_EN: &str = include_str!("atomize_en.txt");
const ATOMIZE_CLI_CS: &str = include_str!("atomize_cli_cs.txt");
const ATOMIZE_CLI_EN: &str = include_str!("atomize_cli_en.txt");
const JUDGE_CS: &str = include_str!("judge_cs.txt");
const JUDGE_EN: &str = include_str!("judge_en.txt");
const JUDGE_CLI_CS: &str = include_str!("judge_cli_cs.txt");
const JUDGE_CLI_EN: &str = include_str!("judge_cli_en.txt");

#[must_use]
pub fn atomize_prompt(locale: &str, provider: ProviderKind) -> &'static str {
    match (provider, locale) {
        (ProviderKind::Anthropic, "cs") => ATOMIZE_CS,
        (ProviderKind::Anthropic, _) => ATOMIZE_EN,
        (ProviderKind::Cli, "cs") => ATOMIZE_CLI_CS,
        (ProviderKind::Cli, _) => ATOMIZE_CLI_EN,
    }
}

#[must_use]
pub fn judge_prompt(locale: &str, provider: ProviderKind) -> &'static str {
    match (provider, locale) {
        (ProviderKind::Anthropic, "cs") => JUDGE_CS,
        (ProviderKind::Anthropic, _) => JUDGE_EN,
        (ProviderKind::Cli, "cs") => JUDGE_CLI_CS,
        (ProviderKind::Cli, _) => JUDGE_CLI_EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomize_prompt_returns_anthropic_czech_for_cs() {
        let p = atomize_prompt("cs", ProviderKind::Anthropic);
        assert!(p.contains("Pracuj v češtině"));
        assert!(p.contains("tool"));
    }

    #[test]
    fn atomize_prompt_returns_anthropic_english_for_en() {
        let p = atomize_prompt("en", ProviderKind::Anthropic);
        assert!(p.contains("Work in English"));
        assert!(p.contains("tool"));
    }

    #[test]
    fn atomize_prompt_cli_cs_instructs_json_only() {
        let p = atomize_prompt("cs", ProviderKind::Cli);
        assert!(p.contains("Pracuj v češtině"));
        assert!(p.contains("JSON"));
        assert!(p.contains("bez markdown"));
    }

    #[test]
    fn atomize_prompt_cli_en_instructs_json_only() {
        let p = atomize_prompt("en", ProviderKind::Cli);
        assert!(p.contains("Work in English"));
        assert!(p.contains("JSON"));
        assert!(p.contains("no markdown"));
    }

    #[test]
    fn atomize_prompt_unknown_locale_falls_back_to_en() {
        assert_eq!(
            atomize_prompt("de", ProviderKind::Anthropic),
            atomize_prompt("en", ProviderKind::Anthropic)
        );
        assert_eq!(
            atomize_prompt("de", ProviderKind::Cli),
            atomize_prompt("en", ProviderKind::Cli)
        );
    }

    #[test]
    fn judge_prompt_non_empty_for_all_combos() {
        for provider in [ProviderKind::Anthropic, ProviderKind::Cli] {
            for locale in ["cs", "en"] {
                assert!(!judge_prompt(locale, provider).trim().is_empty());
            }
        }
    }
}
