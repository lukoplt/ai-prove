use druhy_nazor_lib::llm::anthropic::{AnthropicProvider, DEFAULT_MODEL};
use druhy_nazor_lib::llm::{LlmProvider, RawClaimKind};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Fixture {
    name: String,
    input: String,
    expected_min_claims: usize,
    must_classify_as_fact: Vec<String>,
    must_classify_as_opinion: Vec<String>,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_fixtures() -> Vec<Fixture> {
    let mut out: Vec<Fixture> = Vec::new();
    for entry in fs::read_dir(fixtures_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        out.push(serde_json::from_str(&text).unwrap());
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn provider() -> AnthropicProvider {
    let key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY required");
    let model = std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    AnthropicProvider::new(key, model, "cs".to_string()).expect("provider construction")
}

#[tokio::test]
#[ignore = "requires RUN_LLM_EVAL=1 and a real Anthropic API key"]
async fn llm_eval_suite() {
    if std::env::var("RUN_LLM_EVAL").as_deref() != Ok("1") {
        eprintln!("RUN_LLM_EVAL=1 not set; skipping.");
        return;
    }

    let fixtures = load_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");

    let provider = provider();
    let mut failures: Vec<String> = Vec::new();

    for fixture in &fixtures {
        let result = match provider.atomize(&fixture.input).await {
            Ok(result) => result,
            Err(error) => {
                failures.push(format!("{}: provider error: {error}", fixture.name));
                continue;
            }
        };

        if result.claims.len() < fixture.expected_min_claims {
            failures.push(format!(
                "{}: got {} claims, expected min {}",
                fixture.name,
                result.claims.len(),
                fixture.expected_min_claims
            ));
        }

        for needle in &fixture.must_classify_as_fact {
            let hit = result
                .claims
                .iter()
                .any(|claim| claim.text.contains(needle) && claim.kind == RawClaimKind::Fact);
            if !hit {
                failures.push(format!(
                    "{}: expected fact containing {needle:?}",
                    fixture.name
                ));
            }
        }

        for needle in &fixture.must_classify_as_opinion {
            let hit = result
                .claims
                .iter()
                .any(|claim| claim.text.contains(needle) && claim.kind == RawClaimKind::Opinion);
            if !hit {
                failures.push(format!(
                    "{}: expected opinion containing {needle:?}",
                    fixture.name
                ));
            }
        }
    }

    let max_failed = fixtures.len().div_ceil(5);
    eprintln!(
        "eval: {}/{} fixtures clean",
        fixtures.len().saturating_sub(failures.len()),
        fixtures.len()
    );
    for failure in &failures {
        eprintln!("  x {failure}");
    }
    assert!(failures.len() <= max_failed, "eval threshold not met");
}
