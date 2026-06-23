use crate::error::ApiResponse;
use crate::services::volume;

#[tauri::command]
pub fn volume_get_c_drive() -> ApiResponse<crate::models::VolumeInfo> {
    match volume::get_c_drive_volume() {
        Ok(data) => ApiResponse::ok(data),
        Err(error) => ApiResponse::err(error),
    }
}
