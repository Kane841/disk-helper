use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use chrono::{DateTime, Utc};
use jwalk::WalkDir;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

use crate::db;
use crate::error::{err, AppError, ErrorCode};

pub(crate) const BATCH_SIZE: usize = 500;

#[derive(Debug, Clone, Serialize)]
pub struct ScanProgressEvent {
    pub scan_run_id: String,
    pub percent: u32,
    pub scanned_files: u64,
    pub skipped_files: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanCompletedEvent {
    pub scan_run_id: String,
    pub status: String,
    pub scanned_files: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ScanSnapshot {
    pub scan_run_id: Option<String>,
    pub status: String,
    pub progress_percent: u32,
    pub scanned_files: u64,
    pub skipped_files: u64,
    pub last_completed_at: Option<String>,
}

impl Default for ScanSnapshot {
    fn default() -> Self {
        Self {
            scan_run_id: None,
            status: "idle".into(),
            progress_percent: 0,
            scanned_files: 0,
            skipped_files: 0,
            last_completed_at: None,
        }
    }
}

pub struct ScanCallbacks {
    pub on_progress: Option<Arc<dyn Fn(ScanProgressEvent) + Send + Sync>>,
    pub on_completed: Option<Arc<dyn Fn(ScanCompletedEvent) + Send + Sync>>,
}

pub fn scan_root() -> PathBuf {
    std::env::var("SCAN_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\"))
}

pub fn last_completed_at(conn: &Connection) -> Result<Option<String>, AppError> {
    conn.query_row(
        "SELECT finished_at FROM scan_run
         WHERE status = 'completed' AND finished_at IS NOT NULL
         ORDER BY finished_at DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(map_sqlite_err)
}

pub(crate) struct FileRow {
    path: String,
    parent_path: String,
    name: String,
    is_dir: bool,
    size_bytes: u64,
    folder_size: u64,
    modified_at: Option<String>,
    extension: Option<String>,
    coverage: String,
}

pub(crate) fn file_row_from_path(path: &Path, metadata: &std::fs::Metadata, coverage: &str) -> FileRow {
    let is_dir = metadata.is_dir();
    let size_bytes = if is_dir { 0 } else { metadata.len() };
    let folder_size = if is_dir { 0 } else { size_bytes };
    let modified_at = metadata.modified().ok().map(system_time_to_rfc3339);
    let extension = if is_dir {
        None
    } else {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
    };

    FileRow {
        path: path_to_string(path),
        parent_path: parent_path(path),
        name: file_name(path),
        is_dir,
        size_bytes,
        folder_size,
        modified_at,
        extension,
        coverage: coverage.into(),
    }
}

pub fn run_full_scan_worker(
    data_dir: &Path,
    scan_root: &Path,
    scan_run_id: &str,
    pause: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
    snapshot: Arc<Mutex<ScanSnapshot>>,
    callbacks: ScanCallbacks,
) -> Result<(), AppError> {
    let started = Instant::now();
    let mut conn = db::open(data_dir).map_err(map_db_err)?;
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut scanned_files = 0u64;
    let mut skipped_files = 0u64;

    let threads = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1).min(8))
        .unwrap_or(1);
    let mut walker = WalkDir::new(scan_root).follow_links(false);
    if !cfg!(test) {
        walker = walker.parallelism(jwalk::Parallelism::RayonNewPool(threads));
    }

    for entry in walker.into_iter() {
        wait_if_paused(&pause, &cancel)?;
        if cancel.load(Ordering::SeqCst) {
            finish_scan(
                &conn,
                scan_run_id,
                "cancelled",
                scanned_files,
                skipped_files,
                started,
                snapshot,
                &callbacks,
            )?;
            return Ok(());
        }

        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path == scan_root && entry.file_type().is_dir() {
                    continue;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        skipped_files += 1;
                        insert_skip(&conn, scan_run_id, &path, &format!("metadata: {e}"))?;
                        continue;
                    }
                };

                let is_dir = metadata.is_dir();
                if !is_dir {
                    scanned_files += 1;
                }

                batch.push(file_row_from_path(&path, &metadata, "full"));

                if batch.len() >= BATCH_SIZE {
                    flush_batch(&mut conn, &batch, scan_run_id)?;
                    batch.clear();
                    emit_progress(
                        scan_run_id,
                        scanned_files,
                        skipped_files,
                        &snapshot,
                        &callbacks,
                    )?;
                }
            }
            Err(error) => {
                skipped_files += 1;
                insert_skip(&conn, scan_run_id, scan_root, &format!("walk: {error}"))?;
            }
        }
    }

    if !batch.is_empty() {
        flush_batch(&mut conn, &batch, scan_run_id)?;
    }

    finish_scan(
        &conn,
        scan_run_id,
        "completed",
        scanned_files,
        skipped_files,
        started,
        snapshot,
        &callbacks,
    )
}

pub fn scan_fixture(root: &Path, data_dir: &Path) -> Result<u64, AppError> {
    let conn = db::open(data_dir).map_err(map_db_err)?;
    let scan_run_id = Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO scan_run (id, scan_type, status, started_at, scanned_files, skipped_files)
         VALUES (?1, 'full', 'running', ?2, 0, 0)",
        params![scan_run_id, started_at],
    )
    .map_err(map_sqlite_err)?;
    conn.execute("DELETE FROM file_entry", [])
        .map_err(map_sqlite_err)?;

    run_full_scan_worker(
        data_dir,
        root,
        &scan_run_id,
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
        Arc::new(Mutex::new(ScanSnapshot::default())),
        ScanCallbacks {
            on_progress: None,
            on_completed: None,
        },
    )?;

    conn.query_row(
        "SELECT scanned_files FROM scan_run WHERE id = ?1",
        params![scan_run_id],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count as u64)
    .map_err(map_sqlite_err)
}

pub(crate) fn wait_if_paused(pause: &AtomicBool, cancel: &AtomicBool) -> Result<(), AppError> {
    while pause.load(Ordering::SeqCst) {
        if cancel.load(Ordering::SeqCst) {
            return Ok(());
        }
        thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

pub(crate) fn flush_batch(conn: &mut Connection, batch: &[FileRow], scan_run_id: &str) -> Result<(), AppError> {
    let tx = conn.transaction().map_err(map_sqlite_err)?;
    for row in batch {
        tx.execute(
            "INSERT INTO file_entry (
                path, parent_path, name, is_dir, size_bytes, folder_size,
                modified_at, extension, coverage, scan_run_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(path) DO UPDATE SET
                parent_path = excluded.parent_path,
                name = excluded.name,
                is_dir = excluded.is_dir,
                size_bytes = excluded.size_bytes,
                folder_size = excluded.folder_size,
                modified_at = excluded.modified_at,
                extension = excluded.extension,
                coverage = excluded.coverage,
                scan_run_id = excluded.scan_run_id",
            params![
                row.path,
                row.parent_path,
                row.name,
                row.is_dir as i64,
                row.size_bytes as i64,
                row.folder_size as i64,
                row.modified_at,
                row.extension,
                row.coverage,
                scan_run_id,
            ],
        )
        .map_err(map_sqlite_err)?;
    }
    tx.commit().map_err(map_sqlite_err)
}

pub(crate) fn insert_skip(
    conn: &Connection,
    scan_run_id: &str,
    path: &Path,
    reason: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO scan_skip (scan_run_id, path, reason) VALUES (?1, ?2, ?3)",
        params![scan_run_id, path_to_string(path), reason],
    )
    .map_err(map_sqlite_err)?;
    Ok(())
}

pub(crate) fn finish_scan(
    conn: &Connection,
    scan_run_id: &str,
    status: &str,
    scanned_files: u64,
    skipped_files: u64,
    started: Instant,
    snapshot: Arc<Mutex<ScanSnapshot>>,
    callbacks: &ScanCallbacks,
) -> Result<(), AppError> {
    let finished_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE scan_run
         SET status = ?1, finished_at = ?2, scanned_files = ?3, skipped_files = ?4
         WHERE id = ?5",
        params![
            status,
            finished_at,
            scanned_files as i64,
            skipped_files as i64,
            scan_run_id,
        ],
    )
    .map_err(map_sqlite_err)?;

    if status == "completed" {
        if let Err(error) = crate::services::index::rebuild_folder_sizes(conn) {
            eprintln!("Failed to rebuild folder sizes: {error}");
        }
    }

    {
        let mut snap = snapshot.lock().expect("scan snapshot lock");
        snap.status = status.into();
        snap.progress_percent = if status == "completed" { 100 } else { snap.progress_percent };
        snap.scanned_files = scanned_files;
        snap.skipped_files = skipped_files;
        if status == "completed" {
            snap.last_completed_at = Some(finished_at);
        }
    }

    if let Some(handler) = &callbacks.on_completed {
        handler(ScanCompletedEvent {
            scan_run_id: scan_run_id.into(),
            status: status.into(),
            scanned_files,
            duration_ms: started.elapsed().as_millis() as u64,
        });
    }

    Ok(())
}

pub(crate) fn emit_progress(
    scan_run_id: &str,
    scanned_files: u64,
    skipped_files: u64,
    snapshot: &Arc<Mutex<ScanSnapshot>>,
    callbacks: &ScanCallbacks,
) -> Result<(), AppError> {
    let percent = progress_percent(scanned_files);
    {
        let mut snap = snapshot.lock().expect("scan snapshot lock");
        snap.progress_percent = percent;
        snap.scanned_files = scanned_files;
        snap.skipped_files = skipped_files;
    }

    if let Some(handler) = &callbacks.on_progress {
        handler(ScanProgressEvent {
            scan_run_id: scan_run_id.into(),
            percent,
            scanned_files,
            skipped_files,
        });
    }
    Ok(())
}

fn progress_percent(scanned_files: u64) -> u32 {
    if scanned_files == 0 {
        return 0;
    }
    ((scanned_files.min(9_900) as f64 / 10_000.0) * 99.0).round() as u32
}

pub(crate) fn normalize_windows_path(path: &str) -> String {
    let mut normalized = path.replace('/', "\\");
    while normalized.contains("\\\\") {
        normalized = normalized.replace("\\\\", "\\");
    }
    normalized
}

pub(crate) fn path_to_string(path: &Path) -> String {
    normalize_windows_path(&path.to_string_lossy())
}

pub(crate) fn parent_path(path: &Path) -> String {
    path.parent()
        .map(path_to_string)
        .unwrap_or_else(|| path_to_string(path))
}

pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path_to_string(path))
}

pub(crate) fn system_time_to_rfc3339(time: std::time::SystemTime) -> String {
    DateTime::<Utc>::from(time).to_rfc3339()
}

pub(crate) fn map_sqlite_err(error: rusqlite::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("database error: {error}"),
    )
    .with_target("scan")
}

pub(crate) fn map_db_err(error: db::DbError) -> AppError {
    err(ErrorCode::InternalError, error.to_string()).with_target("scan")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_tree")
    }

    #[test]
    fn full_scan_counts_files() {
        let temp = tempfile::tempdir().expect("temp dir");
        let count = scan_fixture(&fixture_root(), temp.path()).expect("scan fixture");
        assert_eq!(count, 12);
    }
}
