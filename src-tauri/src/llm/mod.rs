pub mod anthropic;
pub mod mock;
pub mod prompts;

use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawClaim {
    pub text: String,
    pub kind: RawClaimKind,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawClaimKind {
    Fact,
    Inference,
    Opinion,
    Contradiction,
}

impl From<RawClaimKind> for crate::models::ClaimKind {
    fn from(kind: RawClaimKind) -> Self {
        match kind {
            RawClaimKind::Fact => Self::Fact,
            RawClaimKind::Inference => Self::Inference,
            RawClaimKind::Opinion => Self::Opinion,
            RawClaimKind::Contradiction => Self::Contradiction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationResult {
    pub claims: Vec<RawClaim>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Supports,
    Contradicts,
    Mentions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeVerdict {
    pub stance: Stance,
    pub quote: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn atomize(&self, input: &str) -> AppResult<AtomizationResult>;
    async fn judge(&self, claim: &str, source_text: &str) -> AppResult<JudgeVerdict>;
}
