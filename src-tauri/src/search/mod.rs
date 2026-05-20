pub mod brave;
pub mod extract;

use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<SearchResult>>;
}

pub struct MockSearch {
    pub results: Vec<SearchResult>,
}

#[async_trait]
impl SearchProvider for MockSearch {
    async fn search(&self, _query: &str, limit: usize) -> AppResult<Vec<SearchResult>> {
        Ok(self.results.iter().take(limit).cloned().collect())
    }
}
