mod commands;
mod db;
mod error;
mod models;
mod services;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = AppState::new().map_err(|err| {
                eprintln!("Failed to initialize application state: {err}");
                err.to_string()
            })?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::volume::volume_get_c_drive,
            commands::scan::scan_start,
            commands::scan::scan_pause,
            commands::scan::scan_resume,
            commands::scan::scan_cancel,
            commands::scan::scan_get_status,
            commands::index::index_get_category_stats,
            commands::index::index_get_children,
            commands::index::index_search,
            commands::index::index_get_top_files,
            commands::index::index_get_top_folders,
            commands::index::index_clear,
            commands::rules::rules_get_suggestions,
            commands::quarantine::quarantine_list,
            commands::quarantine::quarantine_restore,
            commands::quarantine::quarantine_purge,
            commands::cleanup::cleanup_execute,
            commands::audit::audit_list,
            commands::audit::audit_export,
            commands::audit::audit_clear,
            commands::config::config_get,
            commands::config::config_save,
            commands::ai::ai_chat,
            commands::ai::ai_test_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
