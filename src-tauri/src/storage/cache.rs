use crate::error::AppResult;
use crate::models::Verification;
use crate::storage::db::Db;
use sha2::{Digest, Sha256};

#[must_use]
pub fn hash_claim(text: &str) -> String {
    let normalized = normalize(text);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn get(db: &Db, claim_hash: &str, ttl_ms: i64, now_ms: i64) -> AppResult<Option<Verification>> {
    db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT verification, created_at_ms FROM verification_cache WHERE claim_hash = ?",
        )?;
        let mut rows = stmt.query([claim_hash])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            let created_at_ms: i64 = row.get(1)?;
            if now_ms - created_at_ms <= ttl_ms {
                let verification = serde_json::from_str(&json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                return Ok(Some(verification));
            }
        }

        Ok(None)
    })
}

pub fn put(
    db: &Db,
    claim_hash: &str,
    claim_text: &str,
    verification: &Verification,
    now_ms: i64,
) -> AppResult<()> {
    let json = serde_json::to_string(verification)?;
    db.with(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO verification_cache (claim_hash, claim_text, verification, created_at_ms) VALUES (?,?,?,?)",
            rusqlite::params![claim_hash, claim_text, json, now_ms],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SourceHit, SourceStance, SourceTier, VerificationStatus};

    fn sample_verification() -> Verification {
        Verification {
            status: VerificationStatus::Supported,
            sources: vec![SourceHit {
                url: "https://cs.wikipedia.org/x".into(),
                title: "X".into(),
                snippet: "y".into(),
                tier: SourceTier::A,
                stance: SourceStance::Supports,
            }],
            summary: "OK".into(),
        }
    }

    #[test]
    fn normalize_collapses_and_lowercases() {
        assert_eq!(
            normalize("  Karel  IV.  se  Narodil "),
            "karel iv. se narodil"
        );
    }

    #[test]
    fn hash_is_deterministic_and_normalization_insensitive() {
        let first = hash_claim("Karel IV. se narodil");
        let second = hash_claim("  karel iv.   se narodil  ");

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn put_then_get_within_ttl() {
        let db = Db::open_in_memory().unwrap();
        let hash = hash_claim("c");
        put(&db, &hash, "c", &sample_verification(), 1000).unwrap();

        let got = get(&db, &hash, 7 * 24 * 3_600 * 1000, 2000).unwrap();

        assert!(got.is_some());
    }

    #[test]
    fn get_expires_past_ttl() {
        let db = Db::open_in_memory().unwrap();
        let hash = hash_claim("c");
        put(&db, &hash, "c", &sample_verification(), 0).unwrap();

        let got = get(&db, &hash, 1000, 5000).unwrap();

        assert!(got.is_none());
    }

    #[test]
    fn put_replaces_on_conflict() {
        let db = Db::open_in_memory().unwrap();
        let hash = hash_claim("c");
        let mut first = sample_verification();
        first.summary = "first".into();
        let mut second = sample_verification();
        second.summary = "second".into();

        put(&db, &hash, "c", &first, 1000).unwrap();
        put(&db, &hash, "c", &second, 2000).unwrap();
        let got = get(&db, &hash, 1_000_000, 3000).unwrap().unwrap();

        assert_eq!(got.summary, "second");
    }
}
