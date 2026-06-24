use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::models::cleanup::{CleanupExecuteItem, CleanupFailedItem};
use crate::services::cleanup::{self, paths_from_items};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CleanupExecuteResponse {
    pub success_count: u32,
    pub failed: Vec<CleanupFailedItem>,
}

#[tauri::command]
pub fn cleanup_execute(
    state: State<'_, AppState>,
    items: Vec<CleanupExecuteItem>,
    target: String,
    danger_confirm_token: Option<String>,
) -> ApiResponse<CleanupExecuteResponse> {
    let conn = state.db.lock().expect("db lock");
    let paths = paths_from_items(&items);
    match cleanup::execute(
        &conn,
        &state.data_dir,
        paths,
        &target,
        danger_confirm_token.as_deref(),
    ) {
        Ok(result) => ApiResponse::ok(CleanupExecuteResponse {
            success_count: result.success_count,
            failed: result.failed,
        }),
        Err(error) => ApiResponse::err(error),
    }
}
