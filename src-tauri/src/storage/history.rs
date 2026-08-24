use crate::error::AppResult;
use crate::models::Analysis;
use crate::storage::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// How much of the analysed input a list row carries. Enough to recognise an
/// entry, small enough that listing 50 rows stays cheap.
pub const PREVIEW_CHARS: usize = 160;

/// Default page size for `list_history`.
pub const DEFAULT_LIST_LIMIT: usize = 50;

/// A row in the history list. Deliberately not the whole `Analysis` — the full
/// record is fetched only when the user opens one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: i64,
    pub preview: String,
    pub claim_count: usize,
}

pub fn insert(db: &Db, analysis: &Analysis) -> AppResult<()> {
    let json = serde_json::to_string(analysis)?;
    db.with(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO analysis_history (id, created_at_ms, input, analysis_json) VALUES (?,?,?,?)",
            rusqlite::params![analysis.id, analysis.created_at, analysis.input, json],
        )?;
        Ok(())
    })
}

/// Newest first. `query` does a case-insensitive substring match on the
/// analysed input; `%`, `_`, and `\` in it are treated as literals.
pub fn list(db: &Db, query: Option<&str>, limit: usize) -> AppResult<Vec<HistoryEntry>> {
    let limit = i64::try_from(limit.max(1)).unwrap_or(i64::from(u32::MAX));
    let pattern = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", escape_like(&value.to_lowercase())));

    // A single statement with a nullable `?1` keeps the two cases from drifting
    // apart: when no query is given the filter short-circuits to TRUE.
    let rows: Vec<(String, i64, String, String)> = db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, created_at_ms, input, analysis_json FROM analysis_history \
             WHERE ?1 IS NULL OR lower(input) LIKE ?1 ESCAPE '\\' \
             ORDER BY created_at_ms DESC LIMIT ?2",
        )?;
        let mapped = stmt.query_map(rusqlite::params![pattern, limit], row_tuple)?;
        let mut collected = Vec::new();
        for row in mapped {
            collected.push(row?);
        }
        Ok(collected)
    })?;

    Ok(rows
        .into_iter()
        .map(|(id, created_at, input, json)| HistoryEntry {
            id,
            created_at,
            preview: preview_of(&input),
            claim_count: serde_json::from_str::<Analysis>(&json)
                .map(|analysis| analysis.claims.len())
                .unwrap_or_default(),
        })
        .collect())
}

pub fn get(db: &Db, id: &str) -> AppResult<Option<Analysis>> {
    let json: Option<String> = db.with(|conn| {
        conn.query_row(
            "SELECT analysis_json FROM analysis_history WHERE id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .optional()
    })?;

    match json {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Returns `true` when a row was actually removed.
pub fn delete(db: &Db, id: &str) -> AppResult<bool> {
    let affected = db.with(|conn| {
        conn.execute(
            "DELETE FROM analysis_history WHERE id = ?",
            rusqlite::params![id],
        )
    })?;
    Ok(affected > 0)
}

/// Removes every row. Returns how many were removed.
pub fn clear(db: &Db) -> AppResult<usize> {
    db.with(|conn| conn.execute("DELETE FROM analysis_history", []))
}

/// Removes rows created strictly before `cutoff_ms`. Returns how many.
pub fn prune(db: &Db, cutoff_ms: i64) -> AppResult<usize> {
    db.with(|conn| {
        conn.execute(
            "DELETE FROM analysis_history WHERE created_at_ms < ?",
            rusqlite::params![cutoff_ms],
        )
    })
}

fn row_tuple(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, i64, String, String)> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
}

fn preview_of(input: &str) -> String {
    let normalized = input.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized.chars().take(PREVIEW_CHARS).collect()
}

/// `SQLite` `LIKE` treats `%` and `_` as wildcards. A user searching for "100%"
/// means the literal characters, so escape them and pair with `ESCAPE '\'`.
fn escape_like(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Claim, ClaimKind};

    fn analysis_with(id: &str, created_at: i64, input: &str, claims: usize) -> Analysis {
        Analysis {
            id: id.into(),
            created_at,
            input: input.into(),
            claims: (0..claims)
                .map(|index| Claim {
                    id: format!("c{}", index + 1),
                    text: format!("claim {index}"),
                    start: 0,
                    end: 0,
                    kind: ClaimKind::Fact,
                    reason: String::new(),
                    verification: None,
                })
                .collect(),
            truncated: false,
        }
    }

    fn empty_analysis() -> Analysis {
        analysis_with(
            "01900000-0000-0000-0000-000000000001",
            1_700_000_000_000,
            "hi",
            0,
        )
    }

    #[test]
    fn insert_replaces_on_id_conflict() {
        let db = Db::open_in_memory().unwrap();
        let mut analysis = empty_analysis();
        insert(&db, &analysis).unwrap();
        analysis.input = "again".into();
        insert(&db, &analysis).unwrap();

        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].preview, "again");
    }

    #[test]
    fn list_returns_newest_first_with_claim_counts() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1_000, "older", 2)).unwrap();
        insert(&db, &analysis_with("b", 2_000, "newer", 5)).unwrap();

        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[0].claim_count, 5);
        assert_eq!(entries[1].id, "a");
        assert_eq!(entries[1].claim_count, 2);
    }

    #[test]
    fn list_respects_the_limit() {
        let db = Db::open_in_memory().unwrap();
        for index in 0..5 {
            insert(
                &db,
                &analysis_with(&format!("id{index}"), i64::from(index), "x", 0),
            )
            .unwrap();
        }
        assert_eq!(list(&db, None, 3).unwrap().len(), 3);
    }

    #[test]
    fn list_filters_case_insensitively_on_input() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "Karel IV. se narodil", 0)).unwrap();
        insert(&db, &analysis_with("b", 2, "Praha je hlavni mesto", 0)).unwrap();

        let entries = list(&db, Some("karel"), 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "a");
    }

    #[test]
    fn list_treats_like_wildcards_as_literals() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "100% jistota", 0)).unwrap();
        insert(&db, &analysis_with("b", 2, "neco jineho", 0)).unwrap();

        assert_eq!(list(&db, Some("100%"), 10).unwrap().len(), 1);
        // A bare `%` must not match everything.
        assert_eq!(list(&db, Some("%"), 10).unwrap().len(), 1);
    }

    #[test]
    fn preview_is_truncated_to_the_cap() {
        let db = Db::open_in_memory().unwrap();
        let long = "a".repeat(PREVIEW_CHARS + 50);
        insert(&db, &analysis_with("a", 1, &long, 0)).unwrap();

        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries[0].preview.chars().count(), PREVIEW_CHARS);
    }

    #[test]
    fn preview_collapses_whitespace() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "  first\n\nsecond   third ", 0)).unwrap();

        assert_eq!(
            list(&db, None, 10).unwrap()[0].preview,
            "first second third"
        );
    }

    #[test]
    fn get_roundtrips_the_full_analysis() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "text", 3)).unwrap();

        let loaded = get(&db, "a").unwrap().unwrap();
        assert_eq!(loaded.claims.len(), 3);
        assert_eq!(loaded.input, "text");
        assert!(get(&db, "missing").unwrap().is_none());
    }

    #[test]
    fn delete_removes_one_row_and_reports_whether_it_existed() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "x", 0)).unwrap();

        assert!(delete(&db, "a").unwrap());
        assert!(!delete(&db, "a").unwrap());
        assert!(list(&db, None, 10).unwrap().is_empty());
    }

    #[test]
    fn clear_removes_everything_and_reports_the_count() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("a", 1, "x", 0)).unwrap();
        insert(&db, &analysis_with("b", 2, "y", 0)).unwrap();

        assert_eq!(clear(&db).unwrap(), 2);
        assert!(list(&db, None, 10).unwrap().is_empty());
    }

    #[test]
    fn prune_drops_only_rows_older_than_the_cutoff() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &analysis_with("old", 1_000, "x", 0)).unwrap();
        insert(&db, &analysis_with("new", 5_000, "y", 0)).unwrap();

        assert_eq!(prune(&db, 3_000).unwrap(), 1);
        let entries = list(&db, None, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "new");
    }
}
