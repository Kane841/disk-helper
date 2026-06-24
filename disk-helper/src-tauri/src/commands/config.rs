use tauri::State;

use crate::error::ApiResponse;
use crate::services::config::{self, AppSettings, ConfigSaveInput};
use crate::state::AppState;

#[tauri::command]
pub fn config_get(state: State<'_, AppState>) -> ApiResponse<AppSettings> {
    let conn = state.db.lock().expect("db lock");
    match config::get(&conn, &state.data_dir) {
        Ok(settings) => ApiResponse::ok(settings),
        Err(error) => ApiResponse::err(error),
    }
}

#[tauri::command]
pub fn config_save(
    state: State<'_, AppState>,
    input: ConfigSaveInput,
) -> ApiResponse<AppSettings> {
    let conn = state.db.lock().expect("db lock");
    match config::save(&conn, &state.data_dir, input) {
        Ok(settings) => ApiResponse::ok(settings),
        Err(error) => ApiResponse::err(error),
    }
}
