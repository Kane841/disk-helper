use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::models::index::{CategoryStat, FileNode};
use crate::services::index;
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
