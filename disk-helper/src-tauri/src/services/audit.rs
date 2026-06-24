use std::fs;
use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{err, AppError, ErrorCode};
use crate::models::cleanup::AuditLogDto;

pub fn append(
    conn: &Connection,
    event_type: &str,
    summary: &str,
    result: &str,
    related_path: Option<&str>,
    detail_json: Option<String>,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    let occurred_at = Utc::now().to_rfc3339();
    let detail = detail_json.or_else(|| {
        related_path.map(|path| serde_json::json!({ "related_path": path }).to_string())
    });

    conn.execute(
        "INSERT INTO audit_log (id, occurred_at, event_type, summary, result, detail_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, occurred_at, event_type, summary, result, detail],
    )
    .map_err(map_sqlite_err)?;

    Ok(id)
}

pub fn list(
    conn: &Connection,
    event_type: Option<&str>,
    keyword: Option<&str>,
    page: u32,
    size: u32,
) -> Result<(Vec<AuditLogDto>, u64), AppError> {
    let (items, total) = if event_type.filter(|s| !s.is_empty() && *s != "all").is_some()
        || keyword.map(str::trim).filter(|s| !s.is_empty()).is_some()
    {
        list_filtered(conn, event_type, keyword, page, size)?
    } else {
        list_all(conn, page, size)?
    };
    Ok((items, total))
}

fn list_all(conn: &Connection, page: u32, size: u32) -> Result<(Vec<AuditLogDto>, u64), AppError> {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
        .map_err(map_sqlite_err)?;
    let page = page.max(1);
    let size = size.clamp(1, 500);
    let offset = (page - 1) * size;

    let mut stmt = conn
        .prepare(
            "SELECT id, occurred_at, event_type, summary, result, detail_json
             FROM audit_log
             ORDER BY occurred_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(map_sqlite_err)?;

    let rows = stmt
        .query_map(params![size as i64, offset as i64], map_row)
        .map_err(map_sqlite_err)?;

    let items = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)?;
    Ok((items, total as u64))
}

fn list_filtered(
    conn: &Connection,
    event_type: Option<&str>,
    keyword: Option<&str>,
    page: u32,
    size: u32,
) -> Result<(Vec<AuditLogDto>, u64), AppError> {
    let mut sql =
        String::from("SELECT id, occurred_at, event_type, summary, result, detail_json FROM audit_log");
    let mut conditions = Vec::new();
    let mut bind: Vec<String> = Vec::new();

    if let Some(value) = event_type.filter(|s| !s.is_empty() && *s != "all") {
        conditions.push("event_type = ?");
        bind.push(value.to_string());
    }
    if let Some(value) = keyword.map(str::trim).filter(|s| !s.is_empty()) {
        conditions.push("(summary LIKE ? ESCAPE '\\' OR detail_json LIKE ? ESCAPE '\\')");
        let pattern = format!("%{}%", value.replace('%', "\\%").replace('_', "\\_"));
        bind.push(pattern.clone());
        bind.push(pattern);
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let count_sql = sql.replace(
        "SELECT id, occurred_at, event_type, summary, result, detail_json",
        "SELECT COUNT(*)",
    );
    let total: i64 = conn
        .query_row(&count_sql, rusqlite::params_from_iter(bind.iter()), |row| {
            row.get(0)
        })
        .map_err(map_sqlite_err)?;

    sql.push_str(" ORDER BY occurred_at DESC LIMIT ? OFFSET ?");
    let page = page.max(1);
    let size = size.clamp(1, 500);
    let offset = (page - 1) * size;

    let mut stmt = conn.prepare(&sql).map_err(map_sqlite_err)?;
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = bind
        .into_iter()
        .map(|value| Box::new(value) as Box<dyn rusqlite::ToSql>)
        .collect();
    args.push(Box::new(size as i64));
    args.push(Box::new(offset as i64));

    let rows = stmt
        .query_map(rusqlite::params_from_iter(args.iter().map(|b| b.as_ref())), map_row)
        .map_err(map_sqlite_err)?;

    let items = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_err)?;
    Ok((items, total as u64))
}

pub fn export(
    conn: &Connection,
    data_dir: &Path,
    format: &str,
    limit: u32,
) -> Result<String, AppError> {
    let (items, _) = list(conn, None, None, 1, limit)?;
    let exports_dir = data_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(map_io_err)?;

    let file_name = format!(
        "audit-export-{}.{}",
        Utc::now().format("%Y%m%d-%H%M%S"),
        if format == "txt" { "txt" } else { "json" }
    );
    let file_path = exports_dir.join(&file_name);

    let content = if format == "txt" {
        items
            .iter()
            .map(|item| {
                format!(
                    "[{}] {} | {} | {}",
                    item.occurred_at, item.event_type, item.result, item.summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        serde_json::to_string_pretty(&items).map_err(map_json_err)?
    };

    fs::write(&file_path, content).map_err(map_io_err)?;
    Ok(file_path.to_string_lossy().to_string())
}

pub fn clear(conn: &Connection, confirmed: bool) -> Result<u32, AppError> {
    if !confirmed {
        return Err(
            err(ErrorCode::BadArgument, "请确认清空操作").with_target("audit"),
        );
    }
    let deleted = conn
        .execute("DELETE FROM audit_log", [])
        .map_err(map_sqlite_err)? as u32;
    Ok(deleted)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AuditLogDto> {
    let detail_json: Option<String> = row.get(5)?;
    let related_path = detail_json.as_ref().and_then(|json| {
        serde_json::from_str::<serde_json::Value>(json)
            .ok()
            .and_then(|value| {
                value
                    .get("related_path")
                    .and_then(|path| path.as_str())
                    .map(str::to_string)
            })
    });

    Ok(AuditLogDto {
        id: row.get(0)?,
        occurred_at: row.get(1)?,
        event_type: row.get(2)?,
        summary: row.get(3)?,
        result: row.get(4)?,
        related_path,
    })
}

fn map_sqlite_err(error: rusqlite::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("database error: {error}"),
    )
    .with_target("audit")
}

fn map_io_err(error: std::io::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("IO error: {error}"),
    )
    .with_target("audit")
}

fn map_json_err(error: serde_json::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("json error: {error}"),
    )
    .with_target("audit")
}
