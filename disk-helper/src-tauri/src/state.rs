use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::{self, DbError};
use crate::services::scan::ScanController;

/// Shared application state managed by Tauri (DB + data directory).
pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
    pub scan: ScanController,
}

impl AppState {
    pub fn new() -> Result<Self, DbError> {
        let data_dir = db::default_data_dir()?;
        let conn = db::open(&data_dir)?;
        Ok(Self {
            db: Mutex::new(conn),
            data_dir,
            scan: ScanController::new(),
        })
    }

    #[cfg(test)]
    pub fn new_in(dir: &std::path::Path) -> Result<Self, DbError> {
        let conn = db::open(dir)?;
        Ok(Self {
            db: Mutex::new(conn),
            data_dir: dir.to_path_buf(),
            scan: ScanController::new(),
        })
    }
}
