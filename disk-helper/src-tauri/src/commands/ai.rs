use serde::Serialize;
use tauri::State;

use crate::error::ApiResponse;
use crate::services::ai::{AiService, ContextItem};
use crate::services::config::AiMode;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AiChatResponse {
    pub message: String,
    pub disclaimer: String,
}

#[derive(Debug, Serialize)]
pub struct AiTestConnectionResponse {
    pub status: String,
    pub message: String,
    pub provider: String,
}

#[tauri::command]
pub fn ai_chat(
    state: State<'_, AppState>,
    question: String,
    context_items: Vec<ContextItem>,
) -> ApiResponse<AiChatResponse> {
    let conn = state.db.lock().expect("db lock");
    match AiService::chat(&conn, &state.data_dir, &question, context_items) {
        Ok(result) => ApiResponse::ok(AiChatResponse {
            message: result.message,
            disclaimer: result.disclaimer,
        }),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn ai_test_connection(
    state: State<'_, AppState>,
    ai_mode: Option<AiMode>,
    api_key: Option<String>,
) -> ApiResponse<AiTestConnectionResponse> {
    let conn = state.db.lock().expect("db lock");
    match AiService::test_connection(&conn, &state.data_dir, ai_mode, api_key) {
        Ok(result) => ApiResponse::ok(AiTestConnectionResponse {
            status: result.status,
            message: result.message,
            provider: result.provider,
        }),
        Err(error) => ApiResponse::err(error),
    }
}
