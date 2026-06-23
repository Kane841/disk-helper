use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::error::{err, AppError, ErrorCode};

use super::engine::{
    run_full_scan_worker, scan_root, ScanCallbacks, ScanCompletedEvent, ScanProgressEvent,
    ScanSnapshot,
};
use super::incremental::run_incremental_scan_worker;

pub struct ScanController {
    pause: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    snapshot: Arc<Mutex<ScanSnapshot>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl ScanController {
    pub fn new() -> Self {
        Self {
            pause: Arc::new(AtomicBool::new(false)),
            cancel: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            snapshot: Arc::new(Mutex::new(ScanSnapshot::default())),
            worker: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> ScanSnapshot {
        self.snapshot.lock().expect("scan snapshot lock").clone()
    }

    pub fn start_full(
        &self,
        app: AppHandle,
        db: &Mutex<Connection>,
        data_dir: &Path,
    ) -> Result<String, AppError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(
                err(ErrorCode::ScanAlreadyRunning, "扫描进行中，请稍候").with_target("scan"),
            );
        }

        self.pause.store(false, Ordering::SeqCst);
        self.cancel.store(false, Ordering::SeqCst);

        let scan_run_id = Uuid::new_v4().to_string();
        let started_at = Utc::now().to_rfc3339();

        {
            let conn = db.lock().expect("db lock");
            conn.execute(
                "INSERT INTO scan_run (id, scan_type, status, started_at, scanned_files, skipped_files)
                 VALUES (?1, 'full', 'running', ?2, 0, 0)",
                params![scan_run_id, started_at],
            )
            .map_err(|e| err(ErrorCode::InternalError, e.to_string()).with_target("scan"))?;
            conn.execute("DELETE FROM file_entry", [])
                .map_err(|e| err(ErrorCode::InternalError, e.to_string()).with_target("scan"))?;
            conn.execute("DELETE FROM scan_skip", [])
                .map_err(|e| err(ErrorCode::InternalError, e.to_string()).with_target("scan"))?;
        }

        {
            let mut snap = self.snapshot.lock().expect("scan snapshot lock");
            snap.scan_run_id = Some(scan_run_id.clone());
            snap.status = "running".into();
            snap.progress_percent = 0;
            snap.scanned_files = 0;
            snap.skipped_files = 0;
        }

        let data_dir = data_dir.to_path_buf();
        let pause = Arc::clone(&self.pause);
        let cancel = Arc::clone(&self.cancel);
        let running = Arc::clone(&self.running);
        let snapshot = Arc::clone(&self.snapshot);
        let scan_root_path = scan_root();
        let scan_run_id_for_thread = scan_run_id.clone();

        let handle = thread::spawn(move || {
            let app_for_progress = app.clone();
            let app_for_completed = app;
            let callbacks = ScanCallbacks {
                on_progress: Some(Arc::new(move |event: ScanProgressEvent| {
                    let _ = app_for_progress.emit("scan://progress", event);
                })),
                on_completed: Some(Arc::new(move |event: ScanCompletedEvent| {
                    let _ = app_for_completed.emit("scan://completed", event);
                })),
            };

            if let Err(error) = run_full_scan_worker(
                &data_dir,
                &scan_root_path,
                &scan_run_id_for_thread,
                pause,
                cancel,
                snapshot,
                callbacks,
            ) {
                eprintln!("Scan worker failed: {error}");
            }

            running.store(false, Ordering::SeqCst);
        });

        *self.worker.lock().expect("scan worker lock") = Some(handle);
        Ok(scan_run_id)
    }

    pub fn start_incremental(
        &self,
        app: AppHandle,
        db: &Mutex<Connection>,
        data_dir: &Path,
    ) -> Result<String, AppError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(
                err(ErrorCode::ScanAlreadyRunning, "扫描进行中，请稍候").with_target("scan"),
            );
        }

        {
            let conn = db.lock().expect("db lock");
            if !crate::services::index::index_ready(&conn)? {
                self.running.store(false, Ordering::SeqCst);
                return Err(
                    err(
                        ErrorCode::IndexNotReady,
                        "请先完成一次全盘扫描，再执行增量扫描",
                    )
                    .with_target("scan"),
                );
            }
        }

        self.pause.store(false, Ordering::SeqCst);
        self.cancel.store(false, Ordering::SeqCst);

        let scan_run_id = Uuid::new_v4().to_string();
        let started_at = Utc::now().to_rfc3339();

        {
            let conn = db.lock().expect("db lock");
            conn.execute(
                "INSERT INTO scan_run (id, scan_type, status, started_at, scanned_files, skipped_files)
                 VALUES (?1, 'incremental', 'running', ?2, 0, 0)",
                params![scan_run_id, started_at],
            )
            .map_err(|e| err(ErrorCode::InternalError, e.to_string()).with_target("scan"))?;
        }

        {
            let mut snap = self.snapshot.lock().expect("scan snapshot lock");
            snap.scan_run_id = Some(scan_run_id.clone());
            snap.status = "running".into();
            snap.progress_percent = 0;
            snap.scanned_files = 0;
            snap.skipped_files = 0;
        }

        let data_dir = data_dir.to_path_buf();
        let pause = Arc::clone(&self.pause);
        let cancel = Arc::clone(&self.cancel);
        let running = Arc::clone(&self.running);
        let snapshot = Arc::clone(&self.snapshot);
        let scan_root_path = scan_root();
        let scan_run_id_for_thread = scan_run_id.clone();

        let handle = thread::spawn(move || {
            let app_for_progress = app.clone();
            let app_for_completed = app;
            let callbacks = ScanCallbacks {
                on_progress: Some(Arc::new(move |event: ScanProgressEvent| {
                    let _ = app_for_progress.emit("scan://progress", event);
                })),
                on_completed: Some(Arc::new(move |event: ScanCompletedEvent| {
                    let _ = app_for_completed.emit("scan://completed", event);
                })),
            };

            if let Err(error) = run_incremental_scan_worker(
                &data_dir,
                &scan_root_path,
                &scan_run_id_for_thread,
                pause,
                cancel,
                snapshot,
                callbacks,
            ) {
                eprintln!("Incremental scan worker failed: {error}");
            }

            running.store(false, Ordering::SeqCst);
        });

        *self.worker.lock().expect("scan worker lock") = Some(handle);
        Ok(scan_run_id)
    }

    pub fn pause(&self) -> Result<String, AppError> {
        self.ensure_active()?;
        self.pause.store(true, Ordering::SeqCst);
        self.set_status("paused");
        Ok("paused".into())
    }

    pub fn resume(&self) -> Result<String, AppError> {
        self.ensure_active()?;
        if self.snapshot().status != "paused" {
            return Err(err(ErrorCode::BadArgument, "当前扫描未处于暂停状态").with_target("scan"));
        }
        self.pause.store(false, Ordering::SeqCst);
        self.set_status("running");
        Ok("running".into())
    }

    pub fn cancel(&self) -> Result<String, AppError> {
        self.ensure_active()?;
        self.cancel.store(true, Ordering::SeqCst);
        self.pause.store(false, Ordering::SeqCst);
        Ok("cancelled".into())
    }

    fn ensure_active(&self) -> Result<(), AppError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(err(ErrorCode::BadArgument, "当前没有进行中的扫描").with_target("scan"));
        }
        Ok(())
    }

    fn set_status(&self, status: &str) {
        let mut snap = self.snapshot.lock().expect("scan snapshot lock");
        snap.status = status.into();
    }
}
