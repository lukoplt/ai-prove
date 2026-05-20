use crate::models::SourceTier;
use url::Url;

const A_SUFFIXES: &[&str] = &["wikipedia.org", "wikidata.org", "britannica.com"];
const A_EXACT_DOMAINS: &[&str] = &[
    "ec.europa.eu",
    "europa.eu",
    "who.int",
    "un.org",
    "czso.cz",
    "mzcr.cz",
    "uzis.cz",
    "mzv.cz",
    "mvcr.cz",
    "cnb.cz",
    "ucl.cas.cz",
];
const A_HOST_PATTERNS: &[&str] = &[
    ".gov", ".gov.cz", ".gov.uk", ".gov.au", ".edu", ".ac.uk", ".ac.cz", ".cas.cz",
];
const B_EXACT_DOMAINS: &[&str] = &[
    "ct24.cz",
    "ceskatelevize.cz",
    "irozhlas.cz",
    "rozhlas.cz",
    "novinky.cz",
    "seznamzpravy.cz",
    "idnes.cz",
    "lidovky.cz",
    "denik.cz",
    "ihned.cz",
    "e15.cz",
    "aktualne.cz",
    "hlidacipes.org",
    "respekt.cz",
    "bbc.com",
    "bbc.co.uk",
    "nytimes.com",
    "theguardian.com",
    "ap.org",
    "reuters.com",
    "nature.com",
    "science.org",
    "economist.com",
];
const D_EXACT_DOMAINS: &[&str] = &[
    "facebook.com",
    "twitter.com",
    "x.com",
    "tiktok.com",
    "instagram.com",
    "reddit.com",
    "quora.com",
    "pinterest.com",
];
const D_HOST_PATTERNS: &[&str] = &[
    ".blogspot.",
    ".wordpress.com",
    ".wixsite.com",
    ".weebly.com",
];

#[must_use]
pub fn score(url_str: &str) -> SourceTier {
    let Ok(url) = Url::parse(url_str) else {
        return SourceTier::C;
    };
    let Some(host) = url.host_str() else {
        return SourceTier::C;
    };
    let host = host.trim_start_matches("www.").to_ascii_lowercase();

    if A_EXACT_DOMAINS.iter().any(|domain| host == *domain)
        || A_SUFFIXES.iter().any(|suffix| host.ends_with(suffix))
        || A_HOST_PATTERNS
            .iter()
            .any(|pattern| host.ends_with(pattern) || host.contains(pattern))
    {
        return SourceTier::A;
    }

    if B_EXACT_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
    {
        return SourceTier::B;
    }

    if D_EXACT_DOMAINS.iter().any(|domain| host == *domain)
        || D_HOST_PATTERNS.iter().any(|pattern| host.contains(pattern))
    {
        return SourceTier::D;
    }

    SourceTier::C
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikipedia_is_a() {
        assert_eq!(
            score("https://cs.wikipedia.org/wiki/Karel_IV"),
            SourceTier::A
        );
        assert_eq!(
            score("https://en.wikipedia.org/wiki/Charles_IV"),
            SourceTier::A
        );
    }

    #[test]
    fn czso_is_a() {
        assert_eq!(
            score("https://www.czso.cz/csu/czso/pocet-obyvatel"),
            SourceTier::A
        );
    }

    #[test]
    fn gov_suffix_is_a() {
        assert_eq!(score("https://example.gov"), SourceTier::A);
        assert_eq!(score("https://nara.gov"), SourceTier::A);
        assert_eq!(score("https://mzv.gov.cz"), SourceTier::A);
    }

    #[test]
    fn edu_suffix_is_a() {
        assert_eq!(score("https://mit.edu"), SourceTier::A);
    }

    #[test]
    fn major_news_is_b() {
        assert_eq!(score("https://ct24.ceskatelevize.cz/foo"), SourceTier::B);
        assert_eq!(score("https://www.bbc.com/news/x"), SourceTier::B);
    }

    #[test]
    fn social_is_d() {
        assert_eq!(score("https://twitter.com/x"), SourceTier::D);
        assert_eq!(score("https://reddit.com/r/x"), SourceTier::D);
    }

    #[test]
    fn blogspot_pattern_is_d() {
        assert_eq!(score("https://someone.blogspot.com/post"), SourceTier::D);
    }

    #[test]
    fn unknown_is_c() {
        assert_eq!(score("https://example.com/foo"), SourceTier::C);
    }

    #[test]
    fn invalid_url_is_c() {
        assert_eq!(score("not a url"), SourceTier::C);
    }
}
