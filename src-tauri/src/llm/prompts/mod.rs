const ATOMIZE_CS: &str = include_str!("atomize_cs.txt");
const ATOMIZE_EN: &str = include_str!("atomize_en.txt");
const JUDGE_CS: &str = include_str!("judge_cs.txt");
const JUDGE_EN: &str = include_str!("judge_en.txt");

#[must_use]
pub fn atomize_prompt(locale: &str) -> &'static str {
    match locale {
        "cs" => ATOMIZE_CS,
        _ => ATOMIZE_EN,
    }
}

#[must_use]
pub fn judge_prompt(locale: &str) -> &'static str {
    match locale {
        "cs" => JUDGE_CS,
        _ => JUDGE_EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomize_prompt_returns_czech_for_cs() {
        assert!(atomize_prompt("cs").contains("Pracuj v češtině"));
    }

    #[test]
    fn atomize_prompt_returns_english_for_en() {
        assert!(atomize_prompt("en").contains("Work in English"));
    }

    #[test]
    fn atomize_prompt_falls_back_to_english_for_unknown() {
        assert_eq!(atomize_prompt("de"), atomize_prompt("en"));
        assert_eq!(atomize_prompt(""), atomize_prompt("en"));
    }

    #[test]
    fn judge_prompt_returns_a_non_empty_string_for_supported_locales() {
        assert!(!judge_prompt("cs").trim().is_empty());
        assert!(!judge_prompt("en").trim().is_empty());
    }
}
