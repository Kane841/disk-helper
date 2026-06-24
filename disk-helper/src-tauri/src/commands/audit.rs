use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::models::cleanup::AuditLogDto;
use crate::services::audit;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AuditListResponse {
    pub items: Vec<AuditLogDto>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct AuditExportResponse {
    pub file_path: String,
}

#[derive(Debug, Serialize)]
pub struct AuditClearResponse {
    pub deleted: u32,
}

#[tauri::command]
pub fn audit_list(
    state: State<'_, AppState>,
    event_type: Option<String>,
    keyword: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
) -> ApiResponse<AuditListResponse> {
    let conn = state.db.lock().expect("db lock");
    match audit::list(
        &conn,
        event_type.as_deref(),
        keyword.as_deref(),
        page.unwrap_or(1),
        size.unwrap_or(100),
    ) {
        Ok((items, total)) => ApiResponse::ok(AuditListResponse { items, total }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn audit_export(
    state: State<'_, AppState>,
    format: Option<String>,
    limit: Option<u32>,
) -> ApiResponse<AuditExportResponse> {
    let conn = state.db.lock().expect("db lock");
    let format = format.unwrap_or_else(|| "json".into());
    match audit::export(&conn, &state.data_dir, &format, limit.unwrap_or(200)) {
        Ok(file_path) => ApiResponse::ok(AuditExportResponse { file_path }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn audit_clear(
    state: State<'_, AppState>,
    confirmed: bool,
) -> ApiResponse<AuditClearResponse> {
    let conn = state.db.lock().expect("db lock");
    match audit::clear(&conn, confirmed) {
        Ok(deleted) => ApiResponse::ok(AuditClearResponse { deleted }),
        Err(error) => ApiResponse::err(error),
    }
}
