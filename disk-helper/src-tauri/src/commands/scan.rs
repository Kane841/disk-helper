use serde::Serialize;
use tauri::{AppHandle, State};

use crate::error::{err, ApiResponse, ErrorCode};
use crate::services::scan;
use crate::state::AppState;

#[tauri::command]
pub fn scan_start(
    app: AppHandle,
    state: State<'_, AppState>,
    scan_type: String,
) -> ApiResponse<ScanStartResponse> {
    match scan_type.as_str() {
        "full" => match state.scan.start_full(app, &state.db, &state.data_dir) {
            Ok(scan_run_id) => ApiResponse::ok(ScanStartResponse { scan_run_id }),
            Err(error) => ApiResponse::err(error),
        },
        "incremental" => match state.scan.start_incremental(app, &state.db, &state.data_dir) {
            Ok(scan_run_id) => ApiResponse::ok(ScanStartResponse { scan_run_id }),
            Err(error) => ApiResponse::err(error),
        },
        _ => ApiResponse::err(
            err(ErrorCode::BadArgument, "scan type 必须为 full 或 incremental")
                .with_target("scan"),
        ),
    }
}

#[derive(Debug, Serialize)]
pub struct ScanStartResponse {
    pub scan_run_id: String,
}

#[derive(Debug, Serialize)]
pub struct ScanStatusResponse {
    pub status: String,
    pub progress_percent: u32,
    pub scanned_files: u64,
    pub skipped_files: u64,
    pub last_completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScanStatusOnlyResponse {
    pub status: String,
}

#[tauri::command]
pub fn scan_pause(state: State<'_, AppState>) -> ApiResponse<ScanStatusOnlyResponse> {
    match state.scan.pause() {
        Ok(status) => ApiResponse::ok(ScanStatusOnlyResponse { status }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn scan_resume(state: State<'_, AppState>) -> ApiResponse<ScanStatusOnlyResponse> {
    match state.scan.resume() {
        Ok(status) => ApiResponse::ok(ScanStatusOnlyResponse { status }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn scan_cancel(state: State<'_, AppState>) -> ApiResponse<ScanStatusOnlyResponse> {
    match state.scan.cancel() {
        Ok(status) => ApiResponse::ok(ScanStatusOnlyResponse { status }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn scan_get_status(state: State<'_, AppState>) -> ApiResponse<ScanStatusResponse> {
    let snap = state.scan.snapshot();
    let last_completed_at = match scan::last_completed_at(&state.db.lock().expect("db lock")) {
        Ok(value) => value.or(snap.last_completed_at),
        Err(error) => return ApiResponse::err(error),
    };

    ApiResponse::ok(ScanStatusResponse {
        status: snap.status,
        progress_percent: snap.progress_percent,
        scanned_files: snap.scanned_files,
        skipped_files: snap.skipped_files,
        last_completed_at,
    })
}
