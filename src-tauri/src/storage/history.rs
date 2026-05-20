use crate::error::AppResult;
use crate::models::Analysis;
use crate::storage::db::Db;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_analysis() -> Analysis {
        Analysis {
            id: "01900000-0000-0000-0000-000000000001".into(),
            created_at: 1_700_000_000_000,
            input: "hi".into(),
            claims: Vec::new(),
            truncated: false,
        }
    }

    #[test]
    fn insert_replaces_on_id_conflict() {
        let db = Db::open_in_memory().unwrap();
        let mut analysis = empty_analysis();
        insert(&db, &analysis).unwrap();
        analysis.input = "again".into();
        insert(&db, &analysis).unwrap();

        let count: i64 = db
            .with(|conn| {
                conn.query_row("SELECT count(*) FROM analysis_history", [], |row| {
                    row.get(0)
                })
            })
            .unwrap();
        assert_eq!(count, 1);

        let stored_input: String = db
            .with(|conn| {
                conn.query_row(
                    "SELECT input FROM analysis_history WHERE id = ?",
                    rusqlite::params![&analysis.id],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(stored_input, "again");
    }
}
