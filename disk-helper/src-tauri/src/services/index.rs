use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{err, AppError, ErrorCode};
use crate::models::index::{CategoryStat, FileNode};
use crate::services::scan::engine::normalize_windows_path;

const ALL_CATEGORIES: [&str; 6] = [
    "system",
    "program",
    "user_doc",
    "cache_temp",
    "download",
    "other",
];

const DEFAULT_SCOPE: &str = r"C:\";
const MAX_FOLDER_DEPTH: usize = 64;

pub fn index_ready(conn: &Connection) -> Result<bool, AppError> {
    conn.query_row(
        "SELECT 1 FROM scan_run WHERE status = 'completed' LIMIT 1",
        [],
        |_| Ok(true),
    )
    .optional()
    .map(|value| value.unwrap_or(false))
    .map_err(map_sqlite_err)
}

pub fn rebuild_folder_sizes(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "UPDATE file_entry SET folder_size = size_bytes WHERE is_dir = 0",
        [],
    )
    .map_err(map_sqlite_err)?;

    for _ in 0..MAX_FOLDER_DEPTH {
        conn.execute(
            "UPDATE file_entry SET folder_size = (
                SELECT COALESCE(SUM(c.folder_size), 0)
                FROM file_entry AS c
                WHERE c.parent_path = file_entry.path
            )
            WHERE is_dir = 1",
            [],
        )
        .map_err(map_sqlite_err)?;
    }

    Ok(())
}

pub fn get_category_stats(conn: &Connection) -> Result<Vec<CategoryStat>, AppError> {
    ensure_index_ready(conn)?;

    let mut totals = std::collections::HashMap::<&str, u64>::new();
    for code in ALL_CATEGORIES {
        totals.insert(code, 0);
    }

    let mut stmt = conn
        .prepare(
            "SELECT path, size_bytes FROM file_entry WHERE is_dir = 0",
        )
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        })
        .map_err(map_sqlite_err)?;

    for row in rows {
        let (path, size_bytes) = row.map_err(map_sqlite_err)?;
        let code = classify_path(&path);
        *totals.get_mut(code).expect("category") += size_bytes;
    }

    let total_bytes: u64 = totals.values().sum();
    Ok(ALL_CATEGORIES
        .iter()
        .map(|code| {
            let size_bytes = *totals.get(code).unwrap_or(&0);
            let ratio = if total_bytes == 0 {
                0.0
            } else {
                size_bytes as f64 / total_bytes as f64
            };
            CategoryStat {
                code: (*code).into(),
                size_bytes,
                ratio,
            }
        })
        .collect())
}

pub fn get_children(
    conn: &Connection,
    path: &str,
    sort: &str,
) -> Result<Vec<FileNode>, AppError> {
    ensure_index_ready(conn)?;
    let normalized = normalize_path(path);
    let order_sql = if sort == "name" {
        "ORDER BY is_dir DESC, name COLLATE NOCASE ASC"
    } else {
        "ORDER BY is_dir DESC, folder_size DESC, name COLLATE NOCASE ASC"
    };
    let sql = format!(
        "SELECT path, name, is_dir, size_bytes, folder_size, modified_at, extension, coverage
         FROM file_entry
         WHERE parent_path = ?1
         {order_sql}"
    );

    let mut stmt = conn.prepare(&sql).map_err(map_sqlite_err)?;
    let rows = stmt
        .query_map(params![normalized], row_to_file_node)
        .map_err(map_sqlite_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)
}

pub fn search(conn: &Connection, keyword: &str, limit: u32) -> Result<Vec<FileNode>, AppError> {
    ensure_index_ready(conn)?;
    let trimmed = keyword.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let pattern = format!("%{}%", trimmed.replace('%', "\\%").replace('_', "\\_"));
    let mut stmt = conn
        .prepare(
            "SELECT path, name, is_dir, size_bytes, folder_size, modified_at, extension, coverage
             FROM file_entry
             WHERE path LIKE ?1 ESCAPE '\\' OR name LIKE ?1 ESCAPE '\\'
             ORDER BY size_bytes DESC
             LIMIT ?2",
        )
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map(params![pattern, limit as i64], row_to_file_node)
        .map_err(map_sqlite_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)
}

pub fn get_top_files(
    conn: &Connection,
    scope_path: &str,
    limit: u32,
) -> Result<Vec<FileNode>, AppError> {
    ensure_index_ready(conn)?;
    let scope = normalize_scope(scope_path);
    let pattern = format!("{scope}%");

    let mut stmt = conn
        .prepare(
            "SELECT path, name, is_dir, size_bytes, folder_size, modified_at, extension, coverage
             FROM file_entry
             WHERE is_dir = 0 AND path LIKE ?1
             ORDER BY size_bytes DESC
             LIMIT ?2",
        )
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map(params![pattern, limit as i64], row_to_file_node)
        .map_err(map_sqlite_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)
}

pub fn get_top_folders(
    conn: &Connection,
    scope_path: &str,
    limit: u32,
) -> Result<Vec<FileNode>, AppError> {
    ensure_index_ready(conn)?;
    let scope = normalize_scope(scope_path);
    let pattern = format!("{scope}%");

    let mut stmt = conn
        .prepare(
            "SELECT path, name, is_dir, size_bytes, folder_size, modified_at, extension, coverage
             FROM file_entry
             WHERE is_dir = 1 AND path LIKE ?1
             ORDER BY folder_size DESC
             LIMIT ?2",
        )
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map(params![pattern, limit as i64], row_to_file_node)
        .map_err(map_sqlite_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)
}

/// Wipe scan index tables. Does not touch quarantine or audit data.
pub fn clear(conn: &Connection) -> Result<u64, AppError> {
    let deleted: i64 = conn
        .query_row("SELECT COUNT(*) FROM file_entry", [], |row| row.get(0))
        .map_err(map_sqlite_err)?;

    conn.execute("DELETE FROM file_entry", [])
        .map_err(map_sqlite_err)?;
    conn.execute("DELETE FROM scan_skip", [])
        .map_err(map_sqlite_err)?;
    conn.execute("DELETE FROM scan_run", [])
        .map_err(map_sqlite_err)?;

    Ok(deleted as u64)
}

fn ensure_index_ready(conn: &Connection) -> Result<(), AppError> {
    if index_ready(conn)? {
        Ok(())
    } else {
        Err(
            err(
                ErrorCode::IndexNotReady,
                "索引尚未就绪，请先完成一次全盘扫描",
            )
            .with_target("index"),
        )
    }
}

fn classify_path(path: &str) -> &'static str {
    let normalized = normalize_windows_path(path);
    let upper = normalized.to_ascii_uppercase();

    if upper.starts_with(r"C:\WINDOWS\") || upper == r"C:\WINDOWS" {
        "system"
    } else if upper.starts_with(r"C:\PROGRAM FILES\")
        || upper.starts_with(r"C:\PROGRAM FILES (X86)\")
    {
        "program"
    } else if upper.contains(r"\DOCUMENTS\")
        || upper.contains(r"\PICTURES\")
        || upper.contains(r"\VIDEOS\")
    {
        "user_doc"
    } else if upper.contains(r"\TEMP\")
        || upper.contains(r"\CACHE\")
        || upper.contains(r"\APPDATA\LOCAL\TEMP\")
    {
        "cache_temp"
    } else if upper.contains(r"\DOWNLOADS\") {
        "download"
    } else {
        "other"
    }
}

fn normalize_path(path: &str) -> String {
    let mut normalized = normalize_windows_path(path);
    if normalized.len() == 2 && normalized.ends_with(':') {
        normalized.push('\\');
    }
    if normalized == "C:" {
        normalized = r"C:\".into();
    }
    normalized
}

fn normalize_scope(scope_path: &str) -> String {
    let mut scope = normalize_path(scope_path);
    if scope.is_empty() {
        scope = DEFAULT_SCOPE.into();
    }
    if !scope.ends_with('\\') {
        scope.push('\\');
    }
    scope
}

fn row_to_file_node(row: &Row<'_>) -> rusqlite::Result<FileNode> {
    Ok(FileNode {
        path: row.get(0)?,
        name: row.get(1)?,
        is_dir: row.get::<_, i64>(2)? != 0,
        size_bytes: row.get::<_, i64>(3)? as u64,
        folder_size: row.get::<_, i64>(4)? as u64,
        modified_at: row.get(5)?,
        extension: row.get(6)?,
        coverage: row.get(7)?,
    })
}

fn map_sqlite_err(error: rusqlite::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("database error: {error}"),
    )
    .with_target("index")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::scan::engine::scan_fixture;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_tree")
    }

    #[test]
    fn rebuild_folder_sizes_sums_nested_dirs() {
        let temp = tempfile::tempdir().expect("temp dir");
        scan_fixture(&fixture_root(), temp.path()).expect("scan");
        let conn = crate::db::open(temp.path()).expect("db");
        rebuild_folder_sizes(&conn).expect("rebuild");

        let dir2_size: i64 = conn
            .query_row(
                "SELECT folder_size FROM file_entry WHERE path LIKE '%dir2' AND is_dir = 1",
                [],
                |row| row.get(0),
            )
            .expect("dir2");
        assert!(dir2_size > 0);

        let sub_size: i64 = conn
            .query_row(
                "SELECT folder_size FROM file_entry WHERE path LIKE '%sub' AND is_dir = 1",
                [],
                |row| row.get(0),
            )
            .expect("sub");
        assert!(sub_size > 0);
    }

    #[test]
    fn get_top_files_returns_largest_first() {
        let temp = tempfile::tempdir().expect("temp dir");
        scan_fixture(&fixture_root(), temp.path()).expect("scan");
        let conn = crate::db::open(temp.path()).expect("db");
        rebuild_folder_sizes(&conn).expect("rebuild");

        let root = fixture_root().to_string_lossy().replace('/', "\\");
        let files = get_top_files(&conn, &root, 5).expect("top files");
        assert!(!files.is_empty());
        assert!(!files[0].is_dir);
        for pair in files.windows(2) {
            assert!(pair[0].size_bytes >= pair[1].size_bytes);
        }
    }

    #[test]
    fn clear_removes_index_tables() {
        let temp = tempfile::tempdir().expect("temp dir");
        scan_fixture(&fixture_root(), temp.path()).expect("scan");
        let conn = crate::db::open(temp.path()).expect("db");
        assert!(index_ready(&conn).expect("ready"));

        let deleted = clear(&conn).expect("clear");
        assert!(deleted > 0);
        assert!(!index_ready(&conn).expect("check"));
    }

    #[test]
    fn get_children_lists_direct_entries() {
        let temp = tempfile::tempdir().expect("temp dir");
        scan_fixture(&fixture_root(), temp.path()).expect("scan");
        let conn = crate::db::open(temp.path()).expect("db");
        rebuild_folder_sizes(&conn).expect("rebuild");

        let root = fixture_root().to_string_lossy().replace('/', "\\");
        let children = get_children(&conn, &root, "size").expect("children");
        assert_eq!(children.len(), 4);
    }
}
