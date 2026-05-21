// Smoke test: verify deterministic logic across the pipeline without hitting
// paid APIs. Live readability fetch is included (free).
//
// Run: cargo run --manifest-path src-tauri/Cargo.toml --example smoke

use druhy_nazor_lib::llm::anthropic::AnthropicProvider;
use druhy_nazor_lib::llm::mock::MockProvider;
use druhy_nazor_lib::llm::prompts::{atomize_prompt, judge_prompt};
use druhy_nazor_lib::llm::{
    AtomizationResult, JudgeVerdict, LlmProvider, RawClaim, RawClaimKind, Stance,
};
use druhy_nazor_lib::models::{
    Claim, ClaimKind, SourceStance, SourceTier, Verification, VerificationStatus,
};
use druhy_nazor_lib::pipeline::atomize::atomize_to_claims;
use druhy_nazor_lib::pipeline::source_tier;
use druhy_nazor_lib::pipeline::verify::VerificationEngine;
use druhy_nazor_lib::search::extract::Extractor;
use druhy_nazor_lib::search::{MockSearch, SearchResult};
use druhy_nazor_lib::storage::cache;
use druhy_nazor_lib::storage::db::Db;
use druhy_nazor_lib::storage::settings_store::Settings;
use std::sync::Arc;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() {
    let mut fails = 0_u32;
    let mut passes = 0_u32;

    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            if $cond {
                println!("  OK : {}", $name);
                passes += 1;
            } else {
                println!("  FAIL: {}", $name);
                fails += 1;
            }
        }};
    }

    println!("\n=== 1. source tier scoring ===");
    let tier_cases = [
        ("https://cs.wikipedia.org/wiki/Karel_IV", SourceTier::A),
        ("https://www.czso.cz/x", SourceTier::A),
        ("https://example.gov", SourceTier::A),
        ("https://mit.edu", SourceTier::A),
        ("https://www.bbc.com/news", SourceTier::B),
        ("https://ct24.ceskatelevize.cz/x", SourceTier::B),
        ("https://example.com", SourceTier::C),
        ("https://twitter.com/x", SourceTier::D),
        ("https://reddit.com/r/x", SourceTier::D),
        ("https://x.blogspot.com/post", SourceTier::D),
        ("not a url", SourceTier::C),
    ];
    for (url, expected) in tier_cases {
        let got = source_tier::score(url);
        check!(format!("{url} -> {got:?}"), got == expected);
    }

    println!("\n=== 2. locale mapping ===");
    let locale_cases = [
        ("cs-CZ", "cs"),
        ("cs_CZ", "cs"),
        ("CS", "cs"),
        ("en-US", "en"),
        ("EN", "en"),
        ("de-DE", "en"),
        ("fr", "en"),
        ("", "en"),
        ("zh-CN", "en"),
    ];
    for (raw, expected) in locale_cases {
        let got = Settings::map_locale(raw);
        check!(format!("map_locale({raw:?}) -> {got}"), got == expected);
    }

    println!("\n=== 3. settings system locale + validation ===");
    let s = Settings::with_system_locale();
    println!("  detected locale: {}", s.locale);
    check!(
        "with_system_locale returns supported locale",
        s.locale == "cs" || s.locale == "en"
    );
    check!("default settings validate ok", s.validate().is_ok());

    let invalid = Settings {
        cache_ttl_days: 0,
        ..s.clone()
    };
    check!("zero ttl rejected", invalid.validate().is_err());

    println!("\n=== 4. prompt routing ===");
    check!(
        "atomize_prompt(cs) contains 'Pracuj v češtině'",
        atomize_prompt("cs").contains("Pracuj v češtině")
    );
    check!(
        "atomize_prompt(en) contains 'Work in English'",
        atomize_prompt("en").contains("Work in English")
    );
    check!(
        "atomize_prompt(de) falls back to en",
        atomize_prompt("de") == atomize_prompt("en")
    );
    check!(
        "judge_prompt(cs) non-empty",
        !judge_prompt("cs").trim().is_empty()
    );
    check!(
        "judge_prompt(en) non-empty",
        !judge_prompt("en").trim().is_empty()
    );

    println!("\n=== 5. atomize -> claims with offset resolution ===");
    let input = "Karel IV. se narodil v roce 1316 v Praze. Byl to skvělý král.";
    let mock = MockProvider::new();
    mock.push_atomize(AtomizationResult {
        claims: vec![
            RawClaim {
                text: "Karel IV. se narodil v roce 1316".into(),
                kind: RawClaimKind::Fact,
                reason: "r1".into(),
            },
            RawClaim {
                text: "v Praze".into(),
                kind: RawClaimKind::Fact,
                reason: "r2".into(),
            },
            RawClaim {
                text: "Byl to skvělý král".into(),
                kind: RawClaimKind::Opinion,
                reason: "r3".into(),
            },
            RawClaim {
                text: "TOTO NEEXISTUJE".into(),
                kind: RawClaimKind::Fact,
                reason: "should be dropped".into(),
            },
        ],
        truncated: false,
    });
    let outcome = atomize_to_claims(&mock, input).await.expect("atomize");
    check!("atomize drops misquoted claim", outcome.claims.len() == 3);
    check!("first claim id is c1", outcome.claims[0].id == "c1");
    check!("third claim id is c3", outcome.claims[2].id == "c3");
    check!(
        "offsets match input substring",
        input[outcome.claims[0].start..outcome.claims[0].end] == outcome.claims[0].text
    );
    check!(
        "opinion classified correctly",
        outcome.claims[2].kind == ClaimKind::Opinion
    );

    println!("\n=== 6. atomize cap at MAX_CLAIMS ===");
    let mock2 = MockProvider::new();
    let many: Vec<RawClaim> = (0..40)
        .map(|_| RawClaim {
            text: "a".into(),
            kind: RawClaimKind::Fact,
            reason: "r".into(),
        })
        .collect();
    mock2.push_atomize(AtomizationResult {
        claims: many,
        truncated: false,
    });
    let bulk_input = "a ".repeat(60);
    let bulk = atomize_to_claims(&mock2, &bulk_input).await.expect("bulk");
    check!("capped at 25 claims", bulk.claims.len() == 25);
    check!("truncated flag set when capped", bulk.truncated);

    println!("\n=== 7. mock judge FIFO ===");
    let jmock = MockProvider::new();
    jmock.push_judge(JudgeVerdict {
        stance: Stance::Supports,
        quote: "q1".into(),
    });
    jmock.push_judge(JudgeVerdict {
        stance: Stance::Contradicts,
        quote: "q2".into(),
    });
    let v1 = jmock.judge("c", "s").await.expect("judge1");
    let v2 = jmock.judge("c", "s").await.expect("judge2");
    let v3 = jmock.judge("c", "s").await.expect("judge3");
    check!(
        "judge fifo first is Supports",
        v1.stance == Stance::Supports
    );
    check!(
        "judge fifo second is Contradicts",
        v2.stance == Stance::Contradicts
    );
    check!(
        "judge fifo exhausted falls back to Mentions",
        v3.stance == Stance::Mentions
    );

    println!("\n=== 8. cache hash + TTL ===");
    let db = Db::open_in_memory().expect("db");
    let h1 = cache::hash_claim("Karel IV. se narodil");
    let h2 = cache::hash_claim("  karel iv.   se narodil  ");
    check!("hash normalization insensitive", h1 == h2);
    check!("hash length 64 (sha256 hex)", h1.len() == 64);

    let sample_verification = Verification {
        status: VerificationStatus::Supported,
        sources: Vec::new(),
        summary: "OK".into(),
    };
    cache::put(&db, &h1, "k", &sample_verification, 1000).expect("put");
    let hit = cache::get(&db, &h1, 7 * 86_400 * 1000, 2000).expect("get hit");
    check!("cache get within TTL hits", hit.is_some());
    let miss = cache::get(&db, &h1, 500, 5000).expect("get miss");
    check!("cache get past TTL misses", miss.is_none());

    println!("\n=== 9. full pipeline with mocked LLM + mocked search ===");
    let llm_full = MockProvider::new();
    llm_full.push_atomize(AtomizationResult {
        claims: vec![
            RawClaim {
                text: "Python vznikl v roce 1991".into(),
                kind: RawClaimKind::Fact,
                reason: "datum".into(),
            },
            RawClaim {
                text: "Je to nejlepší jazyk".into(),
                kind: RawClaimKind::Opinion,
                reason: "subjekt".into(),
            },
        ],
        truncated: false,
    });
    llm_full.push_judge(JudgeVerdict {
        stance: Stance::Supports,
        quote: "Python was created in 1991".into(),
    });

    let pipeline_input = "Python vznikl v roce 1991. Je to nejlepší jazyk.";
    let claims_out = atomize_to_claims(&llm_full, pipeline_input)
        .await
        .expect("atomize");
    check!("pipeline produced 2 claims", claims_out.claims.len() == 2);
    let fact_count = claims_out
        .claims
        .iter()
        .filter(|c| c.kind == ClaimKind::Fact)
        .count();
    let opinion_count = claims_out
        .claims
        .iter()
        .filter(|c| c.kind == ClaimKind::Opinion)
        .count();
    check!("1 fact + 1 opinion", fact_count == 1 && opinion_count == 1);

    let engine = VerificationEngine {
        llm: Arc::new(llm_full),
        search: Arc::new(MockSearch {
            results: vec![SearchResult {
                url: "https://cs.wikipedia.org/wiki/Karel_IV.".into(),
                title: "Karel IV.".into(),
                snippet: "1316".into(),
            }],
        }),
        extractor: Arc::new(Extractor::new().expect("extractor")),
        locale: "cs".into(),
    };

    let fact = claims_out
        .claims
        .iter()
        .find(|c| c.kind == ClaimKind::Fact)
        .expect("fact");
    println!("  verify will fetch live Wikipedia article…");
    let verification = engine.verify(&fact.text).await.expect("verify");
    println!(
        "    status={:?} sources={} summary='{}'",
        verification.status,
        verification.sources.len(),
        verification.summary
    );
    check!(
        "verify returned at least one source",
        !verification.sources.is_empty()
    );
    check!(
        "summary in cs locale",
        verification.summary.contains("Tvrzení")
            || verification.summary.contains("zdroj")
            || verification.summary.contains("konsenzu")
            || verification.summary.contains("Nenašel")
    );
    check!(
        "source tier from wikipedia is A",
        verification.sources[0].tier == SourceTier::A
    );

    println!("\n=== 10. en-locale verify summary ===");
    let llm_en = MockProvider::new();
    llm_en.push_judge(JudgeVerdict {
        stance: Stance::Supports,
        quote: "x".into(),
    });
    let engine_en = VerificationEngine {
        llm: Arc::new(llm_en),
        search: Arc::new(MockSearch {
            results: vec![SearchResult {
                url: "https://cs.wikipedia.org/wiki/Karel_IV.".into(),
                title: "Karel IV.".into(),
                snippet: "1316".into(),
            }],
        }),
        extractor: Arc::new(Extractor::new().expect("extractor")),
        locale: "en".into(),
    };
    let v_en = engine_en.verify("Karel IV").await.expect("verify en");
    println!("    en summary='{}'", v_en.summary);
    check!(
        "en summary uses English wording",
        v_en.summary.contains("source")
            || v_en.summary.contains("disagree")
            || v_en.summary.contains("No sources")
    );

    println!("\n=== 11. anthropic provider construction smoke ===");
    let prov = AnthropicProvider::new(
        "fake-key".into(),
        "claude-haiku-4-5-20251001".into(),
        "cs".into(),
    );
    check!("AnthropicProvider::new ok", prov.is_ok());

    println!("\n=== 12. live readability extraction (Czech Wikipedia) ===");
    let extractor = Extractor::new().expect("extractor");
    match extractor
        .fetch_and_extract("https://cs.wikipedia.org/wiki/Karel_IV.")
        .await
    {
        Ok(body) => {
            let count = body.chars().count();
            println!("    {count} chars extracted");
            check!("extracted body non-empty", count > 100);
            check!("extracted body capped at 3000 chars", count <= 3000);
        }
        Err(e) => {
            println!("    FAIL: {e}");
            fails += 1;
        }
    }

    println!("\n=== 13. settings JSON roundtrip ===");
    let original = Settings::with_system_locale();
    let json = serde_json::to_string(&original).expect("ser");
    let back: Settings = serde_json::from_str(&json).expect("de");
    check!("settings roundtrip equal", original == back);

    println!("\n=== 14. Claim/Verification serde roundtrip ===");
    let claim = Claim {
        id: "c1".into(),
        text: "x".into(),
        start: 0,
        end: 1,
        kind: ClaimKind::Fact,
        reason: "r".into(),
        verification: Some(Verification {
            status: VerificationStatus::Supported,
            sources: vec![druhy_nazor_lib::models::SourceHit {
                url: "https://x".into(),
                title: "t".into(),
                snippet: "s".into(),
                tier: SourceTier::A,
                stance: SourceStance::Supports,
            }],
            summary: "ok".into(),
        }),
    };
    let cj = serde_json::to_string(&claim).expect("ser claim");
    let cback: Claim = serde_json::from_str(&cj).expect("de claim");
    check!("claim roundtrip equal", claim == cback);
    check!(
        "snake_case kind field in JSON",
        cj.contains("\"kind\":\"fact\"")
    );
    check!(
        "snake_case status field in JSON",
        cj.contains("\"status\":\"supported\"")
    );
    check!(
        "snake_case tier field in JSON",
        cj.contains("\"tier\":\"a\"")
    );

    println!("\n=== summary ===");
    println!("  {passes} passed, {fails} failed");
    if fails > 0 {
        std::process::exit(1);
    }
}
