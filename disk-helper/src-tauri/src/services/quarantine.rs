use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{err, AppError, ErrorCode};
use crate::models::cleanup::QuarantineItemDto;

pub const RETENTION_DAYS: i64 = 30;
const PURGE_CONFIRM_TEXT: &str = "永久删除";

pub fn quarantine_root(data_dir: &Path) -> PathBuf {
    data_dir.join("quarantine")
}

pub fn list(conn: &Connection, keyword: Option<&str>) -> Result<Vec<QuarantineItemDto>, AppError> {
    let sql = if keyword.map(str::trim).filter(|s| !s.is_empty()).is_some() {
        "SELECT id, original_path, quarantine_path, size_bytes, moved_at, expires_at, risk
         FROM quarantine_item
         WHERE original_path LIKE ?1 ESCAPE '\\'
         ORDER BY moved_at DESC"
    } else {
        "SELECT id, original_path, quarantine_path, size_bytes, moved_at, expires_at, risk
         FROM quarantine_item
         ORDER BY moved_at DESC"
    };

    let mut stmt = conn.prepare(sql).map_err(map_sqlite_err)?;
    let rows = if let Some(kw) = keyword.map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", kw.replace('%', "\\%").replace('_', "\\_"));
        stmt.query_map(params![pattern], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(map_sqlite_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)
}

pub fn move_to_quarantine(
    conn: &Connection,
    data_dir: &Path,
    original_path: &str,
    risk: &str,
    rule_id: Option<&str>,
) -> Result<String, AppError> {
    let source = Path::new(original_path);
    if !source.exists() {
        return Err(
            err(ErrorCode::NotFound, format!("路径不存在: {original_path}"))
                .with_target("quarantine"),
        );
    }

    let metadata = fs::metadata(source).map_err(map_io_err)?;
    let size_bytes = if metadata.is_dir() {
        folder_size_from_index(conn, original_path).unwrap_or(0)
    } else {
        metadata.len()
    };

    let id = Uuid::new_v4().to_string();
    let moved_at = Utc::now();
    let expires_at = moved_at + Duration::days(RETENTION_DAYS);
    let date_folder = moved_at.format("%Y-%m-%d").to_string();
    let quarantine_dir = quarantine_root(data_dir)
        .join(date_folder)
        .join(&id);
    let payload_dir = quarantine_dir.join("payload");
    fs::create_dir_all(&payload_dir).map_err(map_io_err)?;

    let payload_target = payload_dir.join(
        source
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "payload".into()),
    );

    move_path(source, &payload_target)?;

    let meta = serde_json::json!({
        "id": id,
        "original_path": original_path,
        "size_bytes": size_bytes,
        "moved_at": moved_at.to_rfc3339(),
        "expires_at": expires_at.to_rfc3339(),
        "risk": risk,
        "rule_id": rule_id,
    });
    fs::write(
        quarantine_dir.join("meta.json"),
        serde_json::to_string_pretty(&meta).map_err(map_json_err)?,
    )
    .map_err(map_io_err)?;

    conn.execute(
        "INSERT INTO quarantine_item
         (id, original_path, quarantine_path, size_bytes, moved_at, expires_at, risk, rule_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            original_path,
            quarantine_dir.to_string_lossy().to_string(),
            size_bytes as i64,
            moved_at.to_rfc3339(),
            expires_at.to_rfc3339(),
            risk,
            rule_id,
        ],
    )
    .map_err(map_sqlite_err)?;

    Ok(id)
}

pub fn restore(
    conn: &Connection,
    ids: &[String],
    conflict_strategy: Option<&str>,
) -> Result<(u32, Vec<(String, String)>), AppError> {
    let mut restored = 0u32;
    let mut failed = Vec::new();

    for id in ids {
        match restore_one(conn, id, conflict_strategy) {
            Ok(()) => restored += 1,
            Err(error) => failed.push((id.clone(), error.message)),
        }
    }

    Ok((restored, failed))
}

pub fn purge(
    conn: &Connection,
    ids: &[String],
    confirm_text: &str,
) -> Result<u32, AppError> {
    if confirm_text != PURGE_CONFIRM_TEXT {
        return Err(
            err(ErrorCode::BadArgument, "请输入「永久删除」以继续").with_target("quarantine"),
        );
    }

    let mut purged = 0u32;
    for id in ids {
        if purge_one(conn, id).is_ok() {
            purged += 1;
        }
    }
    Ok(purged)
}

fn restore_one(
    conn: &Connection,
    id: &str,
    conflict_strategy: Option<&str>,
) -> Result<(), AppError> {
    let (original_path, quarantine_path): (String, String) = conn
        .query_row(
            "SELECT original_path, quarantine_path FROM quarantine_item WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| {
            err(ErrorCode::NotFound, format!("隔离项不存在: {id}")).with_target("quarantine")
        })?;

    let payload_dir = Path::new(&quarantine_path).join("payload");
    let payload = single_payload_path(&payload_dir)?;
    let target = Path::new(&original_path);

    if target.exists() {
        match conflict_strategy {
            Some("overwrite") => {
                remove_path_recursive(target)?;
            }
            Some("alternate") => {
                return Err(
                    err(ErrorCode::QuarantineConflict, "目标路径已存在，请指定备用路径")
                        .with_target("quarantine"),
                );
            }
            _ => {
                return Err(
                    err(ErrorCode::QuarantineConflict, "目标路径已存在").with_target("quarantine"),
                );
            }
        }
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(map_io_err)?;
    }

    move_path(&payload, target)?;

    conn.execute("DELETE FROM quarantine_item WHERE id = ?1", params![id])
        .map_err(map_sqlite_err)?;
    let _ = remove_path_recursive(Path::new(&quarantine_path));
    Ok(())
}

fn purge_one(conn: &Connection, id: &str) -> Result<(), AppError> {
    let quarantine_path: String = conn
        .query_row(
            "SELECT quarantine_path FROM quarantine_item WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| {
            err(ErrorCode::NotFound, format!("隔离项不存在: {id}")).with_target("quarantine")
        })?;

    conn.execute("DELETE FROM quarantine_item WHERE id = ?1", params![id])
        .map_err(map_sqlite_err)?;
    let _ = remove_path_recursive(Path::new(&quarantine_path));
    Ok(())
}

fn single_payload_path(payload_dir: &Path) -> Result<PathBuf, AppError> {
    let mut entries = fs::read_dir(payload_dir).map_err(map_io_err)?;
    let first = entries.next().transpose().map_err(map_io_err)?;
    let Some(first) = first else {
        return Err(
            err(ErrorCode::InternalError, "隔离区 payload 为空").with_target("quarantine"),
        );
    };
    Ok(first.path())
}

fn folder_size_from_index(conn: &Connection, path: &str) -> Result<u64, AppError> {
    conn.query_row(
        "SELECT folder_size FROM file_entry WHERE path = ?1",
        params![path],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value as u64)
    .map_err(map_sqlite_err)
}

fn move_path(from: &Path, to: &Path) -> Result<(), AppError> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }

    if from.is_dir() {
        copy_dir_recursive(from, to)?;
        remove_path_recursive(from)?;
    } else {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(map_io_err)?;
        }
        fs::copy(from, to).map_err(map_io_err)?;
        fs::remove_file(from).map_err(map_io_err)?;
    }
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), AppError> {
    fs::create_dir_all(to).map_err(map_io_err)?;
    for entry in fs::read_dir(from).map_err(map_io_err)? {
        let entry = entry.map_err(map_io_err)?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(map_io_err)?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(map_io_err)?;
        }
    }
    Ok(())
}

fn remove_path_recursive(path: &Path) -> Result<(), AppError> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(map_io_err)?;
    } else if path.exists() {
        fs::remove_file(path).map_err(map_io_err)?;
    }
    Ok(())
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuarantineItemDto> {
    let expires_at: String = row.get(5)?;
    Ok(QuarantineItemDto {
        id: row.get(0)?,
        original_path: row.get(1)?,
        quarantine_path: row.get(2)?,
        size_bytes: row.get::<_, i64>(3)? as u64,
        moved_at: row.get(4)?,
        expires_at: expires_at.clone(),
        status: compute_status(&expires_at),
        risk: row.get(6)?,
    })
}

fn compute_status(expires_at: &str) -> String {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(expires_at) else {
        return "active".into();
    };
    let expires = parsed.with_timezone(&Utc);
    let now = Utc::now();
    if expires <= now {
        "expired".into()
    } else if expires - now <= Duration::days(3) {
        "expiring".into()
    } else {
        "active".into()
    }
}

fn map_sqlite_err(error: rusqlite::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("database error: {error}"),
    )
    .with_target("quarantine")
}

fn map_io_err(error: std::io::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("IO error: {error}"),
    )
    .with_target("quarantine")
}

fn map_json_err(error: serde_json::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("json error: {error}"),
    )
    .with_target("quarantine")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purge_requires_confirm_text() {
        let temp = tempfile::tempdir().expect("temp dir");
        let conn = crate::db::open(temp.path()).expect("db");
        let result = purge(&conn, &[], "wrong");
        assert!(result.is_err());
    }
}
