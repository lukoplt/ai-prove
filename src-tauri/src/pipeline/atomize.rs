use crate::error::AppResult;
use crate::llm::{AtomizationResult, LlmProvider, RawClaim};
use crate::models::{Claim, ClaimKind};

pub const MAX_CLAIMS: usize = 25;

#[derive(Debug, Clone, PartialEq)]
pub struct AtomizationOutcome {
    pub claims: Vec<Claim>,
    pub truncated: bool,
}

pub async fn atomize_to_claims(
    provider: &dyn LlmProvider,
    input: &str,
) -> AppResult<AtomizationOutcome> {
    let AtomizationResult {
        claims: raw,
        truncated,
    } = provider.atomize(input).await?;
    let resolved = resolve_offsets(input, raw);
    let truncated = truncated || resolved.len() > MAX_CLAIMS;
    let mut taken: Vec<Claim> = resolved.into_iter().take(MAX_CLAIMS).collect();

    for (index, claim) in taken.iter_mut().enumerate() {
        claim.id = format!("c{}", index + 1);
    }

    Ok(AtomizationOutcome {
        claims: taken,
        truncated,
    })
}

fn resolve_offsets(input: &str, raw: Vec<RawClaim>) -> Vec<Claim> {
    let mut cursor = 0usize;
    let mut out = Vec::with_capacity(raw.len());

    for claim in raw {
        let needle = claim.text.trim().to_string();
        if needle.is_empty() {
            continue;
        }

        if let Some(relative) = input.get(cursor..).and_then(|tail| tail.find(&needle)) {
            let byte_start = cursor + relative;
            let byte_end = byte_start + needle.len();
            cursor = byte_end;
            out.push(to_claim(
                &needle,
                byte_to_char_index(input, byte_start),
                byte_to_char_index(input, byte_end),
                claim,
            ));
            continue;
        }

        if let Some(byte_start) = input.find(&needle) {
            let byte_end = byte_start + needle.len();
            out.push(to_claim(
                &needle,
                byte_to_char_index(input, byte_start),
                byte_to_char_index(input, byte_end),
                claim,
            ));
        }
    }

    out
}

fn byte_to_char_index(input: &str, byte_index: usize) -> usize {
    input[..byte_index].chars().count()
}

fn to_claim(text: &str, start: usize, end: usize, raw: RawClaim) -> Claim {
    Claim {
        id: String::new(),
        text: text.to_string(),
        start,
        end,
        kind: ClaimKind::from(raw.kind),
        reason: raw.reason,
        verification: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::mock::MockProvider;
    use crate::llm::{AtomizationResult, RawClaimKind};

    fn raw(text: &str, kind: RawClaimKind) -> RawClaim {
        RawClaim {
            text: text.into(),
            kind,
            reason: "r".into(),
        }
    }

    fn slice_chars(input: &str, start: usize, end: usize) -> String {
        input.chars().skip(start).take(end - start).collect()
    }

    #[tokio::test]
    async fn resolves_offsets_in_order() {
        let input = "Karel IV. se narodil v roce 1316 v Praze. Byl to skvělý král.";
        let provider = MockProvider::new();
        provider.push_atomize(AtomizationResult {
            claims: vec![
                raw("Karel IV. se narodil v roce 1316", RawClaimKind::Fact),
                raw("v Praze", RawClaimKind::Fact),
                raw("Byl to skvělý král", RawClaimKind::Opinion),
            ],
            truncated: false,
        });

        let out = atomize_to_claims(&provider, input).await.unwrap();

        assert_eq!(out.claims.len(), 3);
        assert_eq!(out.claims[0].id, "c1");
        assert_eq!(out.claims[1].id, "c2");
        assert_eq!(out.claims[2].id, "c3");
        assert!(out.claims[0].start < out.claims[1].start);
        assert!(out.claims[1].start < out.claims[2].start);
        assert_eq!(
            slice_chars(input, out.claims[0].start, out.claims[0].end),
            out.claims[0].text
        );
    }

    #[tokio::test]
    async fn emits_character_offsets_for_non_ascii_text() {
        let input = "Česká republika má rozlohu 78 866 km².";
        let provider = MockProvider::new();
        provider.push_atomize(AtomizationResult {
            claims: vec![raw("má rozlohu 78 866 km²", RawClaimKind::Fact)],
            truncated: false,
        });

        let out = atomize_to_claims(&provider, input).await.unwrap();

        assert_eq!(
            slice_chars(input, out.claims[0].start, out.claims[0].end),
            out.claims[0].text
        );
    }

    #[tokio::test]
    async fn drops_misquoted_claim() {
        let input = "Karel IV. se narodil v roce 1316.";
        let provider = MockProvider::new();
        provider.push_atomize(AtomizationResult {
            claims: vec![
                raw("Karel IV. se narodil v roce 1316", RawClaimKind::Fact),
                raw("Karel IV. se narodil v roce 1500", RawClaimKind::Fact),
            ],
            truncated: false,
        });

        let out = atomize_to_claims(&provider, input).await.unwrap();

        assert_eq!(out.claims.len(), 1);
    }

    #[tokio::test]
    async fn caps_at_max_claims() {
        let input = "a ".repeat(40);
        let provider = MockProvider::new();
        provider.push_atomize(AtomizationResult {
            claims: (0..40).map(|_| raw("a", RawClaimKind::Fact)).collect(),
            truncated: false,
        });

        let out = atomize_to_claims(&provider, &input).await.unwrap();

        assert_eq!(out.claims.len(), MAX_CLAIMS);
        assert!(out.truncated);
    }

    #[tokio::test]
    async fn preserves_truncated_flag_from_provider() {
        let provider = MockProvider::new();
        provider.push_atomize(AtomizationResult {
            claims: Vec::new(),
            truncated: true,
        });

        let out = atomize_to_claims(&provider, "x").await.unwrap();

        assert!(out.truncated);
    }
}
