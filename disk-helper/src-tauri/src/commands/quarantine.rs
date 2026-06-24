use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::models::cleanup::QuarantineItemDto;
use crate::services::quarantine;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct QuarantineListResponse {
    pub items: Vec<QuarantineItemDto>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct QuarantineRestoreResponse {
    pub restored: u32,
    pub failed: Vec<RestoreFailedItem>,
}

#[derive(Debug, Serialize)]
pub struct RestoreFailedItem {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct QuarantinePurgeResponse {
    pub purged_count: u32,
}

#[tauri::command]
pub fn quarantine_list(
    state: State<'_, AppState>,
    keyword: Option<String>,
) -> ApiResponse<QuarantineListResponse> {
    let conn = state.db.lock().expect("db lock");
    match quarantine::list(&conn, keyword.as_deref()) {
        Ok(items) => {
            let total = items.len() as u64;
            ApiResponse::ok(QuarantineListResponse { items, total })
        }
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn quarantine_restore(
    state: State<'_, AppState>,
    ids: Vec<String>,
    conflict_strategy: Option<String>,
) -> ApiResponse<QuarantineRestoreResponse> {
    let conn = state.db.lock().expect("db lock");
    match quarantine::restore(&conn, &ids, conflict_strategy.as_deref()) {
        Ok((restored, failed)) => ApiResponse::ok(QuarantineRestoreResponse {
            restored,
            failed: failed
                .into_iter()
                .map(|(id, reason)| RestoreFailedItem { id, reason })
                .collect(),
        }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn quarantine_purge(
    state: State<'_, AppState>,
    ids: Vec<String>,
    confirm_text: String,
) -> ApiResponse<QuarantinePurgeResponse> {
    let conn = state.db.lock().expect("db lock");
    match quarantine::purge(&conn, &ids, &confirm_text) {
        Ok(purged_count) => ApiResponse::ok(QuarantinePurgeResponse { purged_count }),
        Err(error) => ApiResponse::err(error),
    }
}
