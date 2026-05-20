use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

const MIGRATION_001: &str = include_str!("../../migrations/001_init.sql");

#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path).map_err(|error| map_sqlite(&error))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA synchronous=NORMAL;",
        )
        .map_err(|error| map_sqlite(&error))?;
        conn.execute_batch(MIGRATION_001)
            .map_err(|error| map_sqlite(&error))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory().map_err(|error| map_sqlite(&error))?;
        conn.execute_batch(MIGRATION_001)
            .map_err(|error| map_sqlite(&error))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let guard = self.inner.lock().expect("db mutex poisoned");
        f(&guard).map_err(|error| map_sqlite(&error))
    }
}

fn map_sqlite(error: &rusqlite::Error) -> AppError {
    AppError::Other(format!("sqlite: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_runs_migrations() {
        let db = Db::open_in_memory().unwrap();
        let count: i64 = db
            .with(|conn| {
                conn.query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='verification_cache'",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn insert_and_select_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        db.with(|conn| {
            conn.execute(
                "INSERT INTO verification_cache (claim_hash, claim_text, verification, created_at_ms) VALUES (?,?,?,?)",
                rusqlite::params!["h1", "claim", r#"{"status":"supported","sources":[],"summary":""}"#, 1000_i64],
            )
        })
        .unwrap();

        let count: i64 = db
            .with(|conn| {
                conn.query_row("SELECT count(*) FROM verification_cache", [], |row| {
                    row.get(0)
                })
            })
            .unwrap();

        assert_eq!(count, 1);
    }
}
