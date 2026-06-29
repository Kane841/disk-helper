use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::models::index::{CategoryStat, FileNode};
use crate::services::index;
use crate::services::rules;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CategoryStatsResponse {
    pub categories: Vec<CategoryStat>,
}

#[derive(Debug, Serialize)]
pub struct FileNodesResponse {
    pub nodes: Vec<FileNode>,
}

#[derive(Debug, Serialize)]
pub struct FileItemsResponse {
    pub items: Vec<FileNode>,
}

#[derive(Debug, Serialize)]
pub struct IndexClearResponse {
    pub deleted_entries: u64,
}

#[tauri::command]
pub fn index_clear(
    state: State<'_, AppState>,
    confirmed: bool,
) -> ApiResponse<IndexClearResponse> {
    if !confirmed {
        return ApiResponse::err(
            crate::error::err(
                crate::error::ErrorCode::BadArgument,
                "需要确认后才能清空索引",
            )
            .with_target("index"),
        );
    }

    let snapshot = state.scan.snapshot();
    if snapshot.status == "running" || snapshot.status == "paused" {
        return ApiResponse::err(
            crate::error::err(
                crate::error::ErrorCode::ScanAlreadyRunning,
                "扫描进行中，请完成或取消后再清空索引",
            )
            .with_target("index"),
        );
    }

    let conn = state.db.lock().expect("db lock");
    match index::clear(&conn) {
        Ok(deleted_entries) => {
            rules::invalidate_suggestions_cache(
                &mut state.suggestions_cache.lock().expect("suggestions cache lock"),
            );
            let _ = crate::services::audit::append(
                &conn,
                "index_clear",
                &format!("清空扫描索引，删除 {deleted_entries} 条记录"),
                "info",
                None,
                None,
            );
            ApiResponse::ok(IndexClearResponse { deleted_entries })
        }
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn index_get_category_stats(state: State<'_, AppState>) -> ApiResponse<CategoryStatsResponse> {
    let conn = state.db.lock().expect("db lock");
    match index::get_category_stats(&conn) {
        Ok(categories) => ApiResponse::ok(CategoryStatsResponse { categories }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn index_get_children(
    state: State<'_, AppState>,
    path: String,
    sort: Option<String>,
) -> ApiResponse<FileNodesResponse> {
    let conn = state.db.lock().expect("db lock");
    let sort = sort.unwrap_or_else(|| "size".into());
    match index::get_children(&conn, &path, &sort) {
        Ok(nodes) => ApiResponse::ok(FileNodesResponse { nodes }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn index_search(
    state: State<'_, AppState>,
    keyword: String,
    limit: Option<u32>,
) -> ApiResponse<FileItemsResponse> {
    let conn = state.db.lock().expect("db lock");
    match index::search(&conn, &keyword, limit.unwrap_or(100)) {
        Ok(items) => ApiResponse::ok(FileItemsResponse { items }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn index_get_top_files(
    state: State<'_, AppState>,
    scope_path: Option<String>,
    limit: Option<u32>,
) -> ApiResponse<FileItemsResponse> {
    let conn = state.db.lock().expect("db lock");
    let scope = scope_path.unwrap_or_default();
    match index::get_top_files(&conn, &scope, limit.unwrap_or(100)) {
        Ok(items) => ApiResponse::ok(FileItemsResponse { items }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn index_get_top_folders(
    state: State<'_, AppState>,
    scope_path: Option<String>,
    limit: Option<u32>,
) -> ApiResponse<FileItemsResponse> {
    let conn = state.db.lock().expect("db lock");
    let scope = scope_path.unwrap_or_default();
    match index::get_top_folders(&conn, &scope, limit.unwrap_or(100)) {
        Ok(items) => ApiResponse::ok(FileItemsResponse { items }),
        Err(error) => ApiResponse::err(error),
    }
}
