use crate::error::AppResult;
use crate::llm::{LlmProvider, Stance};
use crate::models::{SourceHit, SourceStance, SourceTier, Verification, VerificationStatus};
use crate::pipeline::source_tier;
use crate::search::extract::Extractor;
use crate::search::{SearchProvider, SearchResult};
use futures::future::join_all;
use std::sync::Arc;
use tracing::{debug, warn};

pub const SEARCH_RESULTS_PER_CLAIM: usize = 5;
pub const MAX_SOURCES_IN_RESULT: usize = 3;
pub const VERIFICATION_QUERY_MAX_CHARS: usize = 120;

pub struct VerificationEngine {
    pub llm: Arc<dyn LlmProvider>,
    pub search: Arc<dyn SearchProvider>,
    pub extractor: Arc<Extractor>,
}

impl VerificationEngine {
    pub async fn verify(&self, claim_text: &str) -> AppResult<Verification> {
        let query = build_query(claim_text);
        debug!(?query, "verifying claim");
        let results = self.search.search(&query, SEARCH_RESULTS_PER_CLAIM).await?;

        if results.is_empty() {
            return Ok(Verification {
                status: VerificationStatus::NotFound,
                sources: Vec::new(),
                summary: "Nenašel jsem žádný zdroj k ověření.".into(),
            });
        }

        let llm = self.llm.clone();
        let extractor = self.extractor.clone();
        let claim = claim_text.to_string();
        let futures = results.into_iter().map(|result| {
            let llm = llm.clone();
            let extractor = extractor.clone();
            let claim = claim.clone();
            async move { judge_one(&extractor, llm.as_ref(), &claim, result).await }
        });
        let mut hits: Vec<SourceHit> = join_all(futures).await.into_iter().flatten().collect();

        hits.sort_by(|a, b| {
            tier_rank(a.tier)
                .cmp(&tier_rank(b.tier))
                .then(stance_rank(a.stance).cmp(&stance_rank(b.stance)))
        });

        let status = aggregate(&hits);
        let summary = summarize(status, &hits);
        let sources = hits.into_iter().take(MAX_SOURCES_IN_RESULT).collect();

        Ok(Verification {
            status,
            sources,
            summary,
        })
    }
}

fn build_query(claim_text: &str) -> String {
    let trimmed = claim_text.trim();
    if trimmed.chars().count() <= VERIFICATION_QUERY_MAX_CHARS {
        return trimmed.to_string();
    }

    let mut output = String::new();
    for (index, ch) in trimmed.chars().enumerate() {
        if index >= VERIFICATION_QUERY_MAX_CHARS && ch.is_whitespace() {
            break;
        }
        output.push(ch);
    }

    output.trim_end().to_string()
}

async fn judge_one(
    extractor: &Extractor,
    llm: &dyn LlmProvider,
    claim: &str,
    result: SearchResult,
) -> Option<SourceHit> {
    let tier = source_tier::score(&result.url);
    let body = match extractor.fetch_and_extract(&result.url).await {
        Ok(body) if !body.trim().is_empty() => body,
        Ok(_) => {
            debug!(url = %result.url, "empty body from readability");
            return Some(SourceHit {
                url: result.url,
                title: result.title,
                snippet: result.snippet,
                tier,
                stance: SourceStance::Mentions,
            });
        }
        Err(error) => {
            warn!(url = %result.url, %error, "fetch/extract failed; skipping");
            return None;
        }
    };

    let verdict = match llm.judge(claim, &body).await {
        Ok(verdict) => verdict,
        Err(error) => {
            warn!(url = %result.url, %error, "judge failed; falling back to mentions");
            return Some(SourceHit {
                url: result.url,
                title: result.title,
                snippet: result.snippet,
                tier,
                stance: SourceStance::Mentions,
            });
        }
    };

    let stance = match verdict.stance {
        Stance::Supports => SourceStance::Supports,
        Stance::Contradicts => SourceStance::Contradicts,
        Stance::Mentions => SourceStance::Mentions,
    };
    let snippet = if verdict.quote.trim().is_empty() {
        result.snippet
    } else {
        verdict.quote
    };

    Some(SourceHit {
        url: result.url,
        title: result.title,
        snippet,
        tier,
        stance,
    })
}

const fn tier_rank(tier: SourceTier) -> u8 {
    match tier {
        SourceTier::A => 0,
        SourceTier::B => 1,
        SourceTier::C => 2,
        SourceTier::D => 3,
    }
}

const fn stance_rank(stance: SourceStance) -> u8 {
    match stance {
        SourceStance::Supports => 0,
        SourceStance::Contradicts => 1,
        SourceStance::Mentions => 2,
    }
}

fn aggregate(hits: &[SourceHit]) -> VerificationStatus {
    if hits.is_empty() {
        return VerificationStatus::NotFound;
    }

    let mut a_supports = false;
    let mut a_contradicts = false;
    let mut b_supports = false;
    let mut b_contradicts = false;

    for hit in hits {
        match (hit.tier, hit.stance) {
            (SourceTier::A, SourceStance::Supports) => a_supports = true,
            (SourceTier::A, SourceStance::Contradicts) => a_contradicts = true,
            (SourceTier::B, SourceStance::Supports) => b_supports = true,
            (SourceTier::B, SourceStance::Contradicts) => b_contradicts = true,
            _ => {}
        }
    }

    if a_supports && a_contradicts {
        return VerificationStatus::NoConsensus;
    }
    if a_supports || (b_supports && !b_contradicts && !a_contradicts) {
        return VerificationStatus::Supported;
    }
    if a_contradicts || (b_contradicts && !b_supports) {
        return VerificationStatus::Contradicted;
    }
    if b_supports && b_contradicts {
        return VerificationStatus::NoConsensus;
    }

    VerificationStatus::NotFound
}

fn summarize(status: VerificationStatus, hits: &[SourceHit]) -> String {
    let count = hits.len();
    match status {
        VerificationStatus::Supported => format!("Tvrzení potvrzuje {count} zdrojů."),
        VerificationStatus::Contradicted => format!("Tvrzení vyvrací {count} zdrojů."),
        VerificationStatus::NoConsensus => "Zdroje se neshodují - bez konsenzu.".into(),
        VerificationStatus::NotFound => {
            "Nenašel jsem zdroje, které by se k tvrzení vyjadřovaly.".into()
        }
        VerificationStatus::NotVerified => "Tvrzení nebylo ověřováno.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(tier: SourceTier, stance: SourceStance) -> SourceHit {
        SourceHit {
            url: "https://example.com".into(),
            title: "t".into(),
            snippet: "s".into(),
            tier,
            stance,
        }
    }

    #[test]
    fn build_query_passes_short_claim_through() {
        assert_eq!(
            build_query("Karel IV. se narodil v roce 1316"),
            "Karel IV. se narodil v roce 1316"
        );
    }

    #[test]
    fn build_query_truncates_long_claim_at_word_boundary() {
        let claim = "a ".repeat(80);
        let query = build_query(&claim);

        assert!(query.chars().count() <= VERIFICATION_QUERY_MAX_CHARS + 1);
        assert!(!query.ends_with(' '));
    }

    #[test]
    fn aggregate_a_supports_wins() {
        let hits = vec![
            hit(SourceTier::A, SourceStance::Supports),
            hit(SourceTier::C, SourceStance::Contradicts),
        ];

        assert_eq!(aggregate(&hits), VerificationStatus::Supported);
    }

    #[test]
    fn aggregate_a_contradicts_wins() {
        let hits = vec![
            hit(SourceTier::A, SourceStance::Contradicts),
            hit(SourceTier::C, SourceStance::Supports),
        ];

        assert_eq!(aggregate(&hits), VerificationStatus::Contradicted);
    }

    #[test]
    fn aggregate_a_split_is_no_consensus() {
        let hits = vec![
            hit(SourceTier::A, SourceStance::Supports),
            hit(SourceTier::A, SourceStance::Contradicts),
        ];

        assert_eq!(aggregate(&hits), VerificationStatus::NoConsensus);
    }

    #[test]
    fn aggregate_b_only_supports() {
        let hits = vec![hit(SourceTier::B, SourceStance::Supports)];

        assert_eq!(aggregate(&hits), VerificationStatus::Supported);
    }

    #[test]
    fn aggregate_b_split_is_no_consensus() {
        let hits = vec![
            hit(SourceTier::B, SourceStance::Supports),
            hit(SourceTier::B, SourceStance::Contradicts),
        ];

        assert_eq!(aggregate(&hits), VerificationStatus::NoConsensus);
    }

    #[test]
    fn aggregate_only_mentions_is_not_found() {
        let hits = vec![hit(SourceTier::C, SourceStance::Mentions)];

        assert_eq!(aggregate(&hits), VerificationStatus::NotFound);
    }

    #[test]
    fn aggregate_empty_is_not_found() {
        assert_eq!(aggregate(&[]), VerificationStatus::NotFound);
    }
}
