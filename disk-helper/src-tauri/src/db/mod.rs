use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

const MIGRATION_001: &str = include_str!("../../migrations/001_init.sql");

pub const EXPECTED_TABLES: [&str; 6] = [
    "app_settings",
    "audit_log",
    "file_entry",
    "quarantine_item",
    "scan_run",
    "scan_skip",
];

#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    MissingDataDir,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Sqlite(err) => write!(f, "SQLite error: {err}"),
            Self::MissingDataDir => write!(f, "could not resolve application data directory"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<std::io::Error> for DbError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

/// `%AppData%/DiskHelper/` on Windows.
pub fn default_data_dir() -> Result<PathBuf, DbError> {
    dirs::data_dir()
        .map(|dir| dir.join("DiskHelper"))
        .ok_or(DbError::MissingDataDir)
}

pub fn ensure_data_dirs(data_dir: &Path) -> Result<(), DbError> {
    for sub in ["", "quarantine", "logs", "exports"] {
        let path = if sub.is_empty() {
            data_dir.to_path_buf()
        } else {
            data_dir.join(sub)
        };
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn open(data_dir: &Path) -> Result<Connection, DbError> {
    ensure_data_dirs(data_dir)?;
    let db_path = data_dir.join("diskhelper.db");
    let conn = Connection::open(db_path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_migration (
            version TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let applied = conn.query_row(
        "SELECT COUNT(1) FROM _schema_migration WHERE version = ?1",
        ["001_init"],
        |row| row.get::<_, i64>(0),
    )?;

    if applied == 0 {
        conn.execute_batch(MIGRATION_001)?;
        conn.execute(
            "INSERT INTO _schema_migration (version, applied_at) VALUES (?1, datetime('now'))",
            ["001_init"],
        )?;
    }

    Ok(())
}

pub fn list_user_tables(conn: &Connection) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master
         WHERE type = 'table'
           AND name NOT LIKE 'sqlite_%'
           AND name NOT LIKE '_schema_%'
         ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let mut tables = Vec::new();
    for row in rows {
        tables.push(row?);
    }
    Ok(tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_migration_creates_all_tables() {
        let temp = tempfile::tempdir().expect("temp dir");
        let conn = open(temp.path()).expect("open db");

        let tables = list_user_tables(&conn).expect("list tables");
        for expected in EXPECTED_TABLES {
            assert!(
                tables.iter().any(|name| name == expected),
                "missing table: {expected}, got: {tables:?}",
            );
        }

        // Idempotent: second open should not fail.
        drop(conn);
        open(temp.path()).expect("re-open db");
    }
}
