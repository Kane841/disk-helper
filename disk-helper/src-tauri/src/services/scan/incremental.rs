use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use jwalk::WalkDir;
use rusqlite::{params, Connection};

use crate::db;
use crate::error::{err, AppError, ErrorCode};
use crate::services::index;

use super::engine::{
    emit_progress, file_row_from_path, finish_scan, flush_batch, insert_skip, map_db_err,
    map_sqlite_err, path_to_string, wait_if_paused, FileRow, BATCH_SIZE, ScanCallbacks,
    ScanSnapshot,
};

struct FsChild {
    path: String,
    is_dir: bool,
    size_bytes: u64,
    modified_at: Option<String>,
}

struct DbChild {
    path: String,
    is_dir: bool,
    size_bytes: u64,
    modified_at: Option<String>,
}

pub fn run_incremental_scan_worker(
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

    if !index::index_ready(&conn)? {
        return Err(
            err(
                ErrorCode::IndexNotReady,
                "请先完成一次全盘扫描，再执行增量扫描",
            )
            .with_target("scan"),
        );
    }

    let root = path_to_string(scan_root);
    let shallow_dirs = collect_shallow_dirs(&conn, &root)?;
    let mut rewalk_roots: HashSet<String> = HashSet::new();
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut scanned_files = 0u64;
    let mut skipped_files = 0u64;

    for shallow_dir in &shallow_dirs {
        wait_if_paused(&pause, &cancel)?;
        if cancel.load(Ordering::SeqCst) {
            return finish_cancelled(
                &conn,
                scan_run_id,
                scanned_files,
                skipped_files,
                started,
                snapshot,
                callbacks,
            );
        }

        let fs_children = match read_fs_children(Path::new(shallow_dir)) {
            Ok(children) => children,
            Err(error) => {
                skipped_files += 1;
                insert_skip(&conn, scan_run_id, Path::new(shallow_dir), &error.to_string())?;
                continue;
            }
        };
        let db_children = load_db_children(&conn, shallow_dir)?;

        let fs_map: HashMap<String, FsChild> = fs_children
            .into_iter()
            .map(|child| (child.path.clone(), child))
            .collect();
        let db_map: HashMap<String, DbChild> = db_children
            .into_iter()
            .map(|child| (child.path.clone(), child))
            .collect();

        for child in fs_map.values() {
            match db_map.get(&child.path) {
                None => {
                    if child.is_dir {
                        rewalk_roots.insert(child.path.clone());
                    } else {
                        scanned_files += 1;
                        push_file_row_from_fs_child(&mut batch, child);
                    }
                }
                Some(db) => {
                    if child.is_dir != db.is_dir {
                        purge_subtree(&conn, &child.path)?;
                        if child.is_dir {
                            rewalk_roots.insert(child.path.clone());
                        } else {
                            scanned_files += 1;
                            push_file_row_from_fs_child(&mut batch, child);
                        }
                    } else if !child.is_dir
                        && (child.size_bytes != db.size_bytes
                            || child.modified_at != db.modified_at)
                    {
                        scanned_files += 1;
                        push_file_row_from_fs_child(&mut batch, child);
                    }
                }
            }
        }

        for db in db_map.values() {
            if !fs_map.contains_key(&db.path) {
                purge_subtree(&conn, &db.path)?;
            }
        }

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

    for subtree_root in rewalk_roots {
        wait_if_paused(&pause, &cancel)?;
        if cancel.load(Ordering::SeqCst) {
            return finish_cancelled(
                &conn,
                scan_run_id,
                scanned_files,
                skipped_files,
                started,
                snapshot,
                callbacks,
            );
        }

        purge_subtree(&conn, &subtree_root)?;
        rewalk_subtree(
            &mut conn,
            Path::new(&subtree_root),
            scan_run_id,
            &pause,
            &cancel,
            &mut scanned_files,
            &mut skipped_files,
            &mut batch,
            &snapshot,
            &callbacks,
        )?;
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

pub fn incremental_scan_fixture(root: &Path, data_dir: &Path) -> Result<(), AppError> {
    use chrono::Utc;
    use uuid::Uuid;

    let conn = db::open(data_dir).map_err(map_db_err)?;
    let scan_run_id = Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO scan_run (id, scan_type, status, started_at, scanned_files, skipped_files)
         VALUES (?1, 'incremental', 'running', ?2, 0, 0)",
        params![scan_run_id, started_at],
    )
    .map_err(map_sqlite_err)?;

    run_incremental_scan_worker(
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
    )
}

fn finish_cancelled(
    conn: &Connection,
    scan_run_id: &str,
    scanned_files: u64,
    skipped_files: u64,
    started: Instant,
    snapshot: Arc<Mutex<ScanSnapshot>>,
    callbacks: ScanCallbacks,
) -> Result<(), AppError> {
    finish_scan(
        conn,
        scan_run_id,
        "cancelled",
        scanned_files,
        skipped_files,
        started,
        snapshot,
        &callbacks,
    )?;
    Ok(())
}

fn collect_shallow_dirs(conn: &Connection, scan_root: &str) -> Result<Vec<String>, AppError> {
    let mut dirs = vec![scan_root.to_string()];
    let mut stmt = conn
        .prepare("SELECT path FROM file_entry WHERE is_dir = 1")
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(map_sqlite_err)?;

    for row in rows {
        let path = row.map_err(map_sqlite_err)?;
        let depth = depth_from_root(&path, scan_root);
        if (1..=2).contains(&depth) && !dirs.contains(&path) {
            dirs.push(path);
        }
    }

    Ok(dirs)
}

fn depth_from_root(path: &str, scan_root: &str) -> usize {
    let path = path_to_string(Path::new(path));
    let root = path_to_string(Path::new(scan_root));
    if path == root {
        return 0;
    }
    if !path.starts_with(&root) {
        return usize::MAX;
    }
    let suffix = path.strip_prefix(&root).unwrap_or("");
    suffix
        .trim_start_matches('\\')
        .split('\\')
        .filter(|part| !part.is_empty())
        .count()
}

fn read_fs_children(dir: &Path) -> Result<Vec<FsChild>, AppError> {
    let entries = fs::read_dir(dir).map_err(|error| {
        err(
            ErrorCode::InternalError,
            format!("read_dir {}: {error}", path_to_string(dir)),
        )
        .with_target("scan")
    })?;

    let mut children = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            err(ErrorCode::InternalError, format!("read_dir entry: {error}")).with_target("scan")
        })?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|error| {
            err(
                ErrorCode::InternalError,
                format!("metadata {}: {error}", path_to_string(&path)),
            )
            .with_target("scan")
        })?;

        children.push(FsChild {
            path: path_to_string(&path),
            is_dir: metadata.is_dir(),
            size_bytes: if metadata.is_dir() {
                0
            } else {
                metadata.len()
            },
            modified_at: metadata
                .modified()
                .ok()
                .map(super::engine::system_time_to_rfc3339),
        });
    }

    Ok(children)
}

fn load_db_children(conn: &Connection, parent_path: &str) -> Result<Vec<DbChild>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT path, is_dir, size_bytes, modified_at
             FROM file_entry
             WHERE parent_path = ?1",
        )
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map(params![parent_path], |row| {
            Ok(DbChild {
                path: row.get(0)?,
                is_dir: row.get::<_, i64>(1)? != 0,
                size_bytes: row.get::<_, i64>(2)? as u64,
                modified_at: row.get(3)?,
            })
        })
        .map_err(map_sqlite_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)
}

fn push_file_row_from_fs_child(batch: &mut Vec<FileRow>, child: &FsChild) {
    let path = PathBuf::from(&child.path);
    let metadata = fs::metadata(&path).expect("metadata for changed file");
    batch.push(file_row_from_path(&path, &metadata, "full"));
}

fn purge_subtree(conn: &Connection, root: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM file_entry WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'",
        params![root, format!("{}\\%", root)],
    )
    .map_err(map_sqlite_err)?;
    Ok(())
}

fn rewalk_subtree(
    conn: &mut Connection,
    subtree_root: &Path,
    scan_run_id: &str,
    pause: &AtomicBool,
    cancel: &AtomicBool,
    scanned_files: &mut u64,
    skipped_files: &mut u64,
    batch: &mut Vec<FileRow>,
    snapshot: &Arc<Mutex<ScanSnapshot>>,
    callbacks: &ScanCallbacks,
) -> Result<(), AppError> {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1).min(8))
        .unwrap_or(1);
    let mut walker = WalkDir::new(subtree_root).follow_links(false);
    if !cfg!(test) {
        walker = walker.parallelism(jwalk::Parallelism::RayonNewPool(threads));
    }

    for entry in walker.into_iter() {
        wait_if_paused(pause, cancel)?;
        if cancel.load(Ordering::SeqCst) {
            return Ok(());
        }

        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path == subtree_root && entry.file_type().is_dir() {
                    continue;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(error) => {
                        *skipped_files += 1;
                        insert_skip(conn, scan_run_id, &path, &format!("metadata: {error}"))?;
                        continue;
                    }
                };

                if !metadata.is_dir() {
                    *scanned_files += 1;
                }

                batch.push(file_row_from_path(&path, &metadata, "full"));

                if batch.len() >= BATCH_SIZE {
                    flush_batch(conn, batch, scan_run_id)?;
                    batch.clear();
                    emit_progress(
                        scan_run_id,
                        *scanned_files,
                        *skipped_files,
                        snapshot,
                        callbacks,
                    )?;
                }
            }
            Err(error) => {
                *skipped_files += 1;
                insert_skip(
                    conn,
                    scan_run_id,
                    subtree_root,
                    &format!("walk: {error}"),
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::scan::engine::scan_fixture;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_tree")
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let target = dst.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_dir_all(&entry.path(), &target)?;
            } else {
                fs::copy(entry.path(), &target)?;
            }
        }
        Ok(())
    }

    fn query_file_size(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT size_bytes FROM file_entry WHERE name = ?1 AND is_dir = 0",
            params![name],
            |row| row.get(0),
        )
        .expect("file size")
    }

    fn file_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM file_entry WHERE name = ?1",
            params![name],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false)
    }

    #[test]
    fn incremental_scan_updates_changed_file() {
        let temp = tempfile::tempdir().expect("temp dir");
        let tree = temp.path().join("tree");
        copy_dir_all(&fixture_root(), &tree).expect("copy fixture");
        scan_fixture(&tree, temp.path()).expect("full scan");

        fs::write(tree.join("file2.txt"), b"updated-by-incremental-scan").expect("write");

        incremental_scan_fixture(&tree, temp.path()).expect("incremental scan");

        let conn = db::open(temp.path()).expect("db");
        assert_eq!(
            query_file_size(&conn, "file2.txt"),
            b"updated-by-incremental-scan".len() as i64
        );
    }

    #[test]
    fn incremental_scan_tombstones_deleted_file() {
        let temp = tempfile::tempdir().expect("temp dir");
        let tree = temp.path().join("tree");
        copy_dir_all(&fixture_root(), &tree).expect("copy fixture");
        scan_fixture(&tree, temp.path()).expect("full scan");

        fs::remove_file(tree.join("file1.txt")).expect("delete file");

        incremental_scan_fixture(&tree, temp.path()).expect("incremental scan");

        let conn = db::open(temp.path()).expect("db");
        assert!(!file_exists(&conn, "file1.txt"));
    }
}
