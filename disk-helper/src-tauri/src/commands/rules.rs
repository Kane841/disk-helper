use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::models::cleanup::CleanupSuggestion;
use crate::services::rules::{self, SuggestionFilters};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SuggestionsResponse {
    pub items: Vec<CleanupSuggestion>,
    pub releasable_bytes: u64,
    pub total: u64,
}

#[tauri::command]
pub fn rules_get_suggestions(
    state: State<'_, AppState>,
    risk_filter: Option<String>,
    category_filter: Option<String>,
    path_keyword: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
) -> ApiResponse<SuggestionsResponse> {
    let conn = state.db.lock().expect("db lock");
    let filters = SuggestionFilters {
        risk_filter: risk_filter.as_deref(),
        category_filter: category_filter.as_deref(),
        path_keyword: path_keyword.as_deref(),
        page: page.unwrap_or(1),
        size: size.unwrap_or(100),
    };

    match rules::get_suggestions(&conn, filters) {
        Ok(result) => ApiResponse::ok(SuggestionsResponse {
            items: result.items,
            releasable_bytes: result.releasable_bytes,
            total: result.total,
        }),
        Err(error) => ApiResponse::err(error),
    }
}
