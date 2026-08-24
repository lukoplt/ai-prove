use crate::error::{AppError, AppResult};
use crate::models::Analysis;
use crate::storage::db::Db;
use crate::storage::history::{self, HistoryEntry, DEFAULT_LIST_LIMIT};
use tauri::State;

/// Hard cap so a malformed frontend call cannot ask for the whole table.
const MAX_LIST_LIMIT: usize = 500;

#[tauri::command]
pub async fn list_history(
    db: State<'_, Db>,
    query: Option<String>,
    limit: Option<usize>,
) -> AppResult<Vec<HistoryEntry>> {
    let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT).clamp(1, MAX_LIST_LIMIT);
    history::list(&db, query.as_deref(), limit)
}

#[tauri::command]
pub async fn get_analysis(db: State<'_, Db>, id: String) -> AppResult<Analysis> {
    history::get(&db, &id)?.ok_or_else(|| AppError::NotFound(format!("analysis {id}")))
}

#[tauri::command]
pub async fn delete_analysis(db: State<'_, Db>, id: String) -> AppResult<()> {
    history::delete(&db, &id)?;
    Ok(())
}

#[tauri::command]
pub async fn clear_history(db: State<'_, Db>) -> AppResult<usize> {
    history::clear(&db)
}
