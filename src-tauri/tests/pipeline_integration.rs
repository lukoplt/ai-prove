use druhy_nazor_lib::llm::mock::MockProvider;
use druhy_nazor_lib::llm::{AtomizationResult, JudgeVerdict, RawClaim, RawClaimKind, Stance};
use druhy_nazor_lib::models::{ClaimKind, VerificationStatus};
use druhy_nazor_lib::pipeline::atomize_to_claims;
use druhy_nazor_lib::pipeline::verify::VerificationEngine;
use druhy_nazor_lib::search::extract::Extractor;
use druhy_nazor_lib::search::{MockSearch, SearchResult};
use std::sync::Arc;

#[tokio::test]
async fn atomize_then_verify_supported_with_live_fetch_when_enabled() {
    let input = "Karel IV. se narodil v roce 1316.";
    let llm = MockProvider::new();
    llm.push_atomize(AtomizationResult {
        claims: vec![RawClaim {
            text: "Karel IV. se narodil v roce 1316".into(),
            kind: RawClaimKind::Fact,
            reason: "Historické datum.".into(),
        }],
        truncated: false,
    });
    llm.push_judge(JudgeVerdict {
        stance: Stance::Supports,
        quote: "1316".into(),
    });

    let outcome = atomize_to_claims(&llm, input).await.unwrap();

    assert_eq!(outcome.claims.len(), 1);
    assert_eq!(outcome.claims[0].kind, ClaimKind::Fact);

    if std::env::var("RUN_NETWORK_TESTS").as_deref() != Ok("1") {
        eprintln!("RUN_NETWORK_TESTS=1 not set; skipping live fetch portion.");
        return;
    }

    let engine = VerificationEngine {
        llm: Arc::new(llm),
        search: Arc::new(MockSearch {
            results: vec![SearchResult {
                url: "https://cs.wikipedia.org/wiki/Karel_IV".into(),
                title: "Karel IV.".into(),
                snippet: "Narodil se v roce 1316.".into(),
            }],
        }),
        extractor: Arc::new(Extractor::new().unwrap()),
    };

    let verification = engine.verify(&outcome.claims[0].text).await.unwrap();

    assert_eq!(verification.status, VerificationStatus::Supported);
    assert!(!verification.sources.is_empty());
}
