use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub kind: ClaimKind,
    pub reason: String,
    pub verification: Option<Verification>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Fact,
    Inference,
    Opinion,
    Contradiction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Verification {
    pub status: VerificationStatus,
    pub sources: Vec<SourceHit>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Supported,
    Contradicted,
    NoConsensus,
    NotFound,
    NotVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceHit {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub tier: SourceTier,
    pub stance: SourceStance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceTier {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStance {
    Supports,
    Contradicts,
    Mentions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: String,
    pub created_at: i64,
    pub input: String,
    pub claims: Vec<Claim>,
    pub truncated: bool,
}
