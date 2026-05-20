use super::{AtomizationResult, JudgeVerdict, LlmProvider, Stance};
use crate::error::AppResult;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Default)]
pub struct MockProvider {
    atomize_queue: Mutex<VecDeque<AtomizationResult>>,
    judge_queue: Mutex<VecDeque<JudgeVerdict>>,
}

impl MockProvider {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_atomize(&self, result: AtomizationResult) {
        self.atomize_queue.lock().unwrap().push_back(result);
    }

    pub fn push_judge(&self, verdict: JudgeVerdict) {
        self.judge_queue.lock().unwrap().push_back(verdict);
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn atomize(&self, _input: &str) -> AppResult<AtomizationResult> {
        Ok(self
            .atomize_queue
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(AtomizationResult {
                claims: Vec::new(),
                truncated: false,
            }))
    }

    async fn judge(&self, _claim: &str, _source_text: &str) -> AppResult<JudgeVerdict> {
        Ok(self
            .judge_queue
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(JudgeVerdict {
                stance: Stance::Mentions,
                quote: String::new(),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{RawClaim, RawClaimKind};

    #[tokio::test]
    async fn fifo_atomize() {
        let provider = MockProvider::new();
        provider.push_atomize(AtomizationResult {
            claims: vec![RawClaim {
                text: "A".into(),
                kind: RawClaimKind::Fact,
                reason: "r".into(),
            }],
            truncated: false,
        });
        provider.push_atomize(AtomizationResult {
            claims: Vec::new(),
            truncated: true,
        });

        let first = provider.atomize("ignored").await.unwrap();
        let second = provider.atomize("ignored").await.unwrap();

        assert_eq!(first.claims.len(), 1);
        assert!(!first.truncated);
        assert!(second.claims.is_empty());
        assert!(second.truncated);
    }

    #[tokio::test]
    async fn empty_returns_empty() {
        let provider = MockProvider::new();
        let result = provider.atomize("x").await.unwrap();
        assert!(result.claims.is_empty());
    }

    #[tokio::test]
    async fn fifo_judge() {
        let provider = MockProvider::new();
        provider.push_judge(JudgeVerdict {
            stance: Stance::Supports,
            quote: "q".into(),
        });

        let first = provider.judge("claim", "source").await.unwrap();
        let second = provider.judge("claim", "source").await.unwrap();

        assert_eq!(first.stance, Stance::Supports);
        assert_eq!(first.quote, "q");
        assert_eq!(second.stance, Stance::Mentions);
    }
}
