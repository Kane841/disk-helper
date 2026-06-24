use std::path::Path;

use crate::error::{err, AppError, ErrorCode};
use crate::models::cleanup::{CleanupFailedItem, CleanupExecuteItem};
use crate::services::{audit, quarantine, rules::RulesEngine};

const DANGER_CONFIRM_TEXT: &str = "确认清理";

pub struct CleanupExecuteResult {
    pub success_count: u32,
    pub failed: Vec<CleanupFailedItem>,
}

pub fn execute(
    conn: &rusqlite::Connection,
    data_dir: &Path,
    paths: Vec<String>,
    target: &str,
    danger_confirm_token: Option<&str>,
) -> Result<CleanupExecuteResult, AppError> {
    if paths.is_empty() {
        return Err(
            err(ErrorCode::BadArgument, "请先勾选要清理的项").with_target("cleanup"),
        );
    }

    if target != "quarantine" {
        return Err(
            err(
                ErrorCode::BadArgument,
                "v1 当前仅支持移入隔离区（quarantine）",
            )
            .with_target("cleanup"),
        );
    }

    let engine = RulesEngine::load()?;
    let has_danger = paths
        .iter()
        .any(|path| engine.match_path_info(path).is_some_and(|rule| rule.risk == "danger"));

    if has_danger && danger_confirm_token != Some(DANGER_CONFIRM_TEXT) {
        return Err(
            err(ErrorCode::DangerNotConfirmed, "请输入「确认清理」以继续")
                .with_target("cleanup"),
        );
    }

    let mut success_count = 0u32;
    let mut failed = Vec::new();
    let mut released_bytes = 0u64;

    for path in paths {
        let rule = engine.match_path_info(&path);
        let risk = rule.as_ref().map(|r| r.risk.as_str()).unwrap_or("caution");
        let rule_id = rule.as_ref().map(|r| r.id.as_str());
        let size = file_size(conn, &path).unwrap_or(0);

        match quarantine::move_to_quarantine(conn, data_dir, &path, risk, rule_id) {
            Ok(_id) => {
                remove_from_index(conn, &path)?;
                released_bytes += size;
                success_count += 1;
            }
            Err(error) => failed.push(CleanupFailedItem {
                path,
                reason: error.message,
            }),
        }
    }

    if success_count > 0 {
        let summary = format!("移入隔离区 {success_count} 项，释放约 {released_bytes} 字节");
        let result = if failed.is_empty() {
            "success"
        } else {
            "partial"
        };
        audit::append(conn, "soft_delete", &summary, result, None, None)?;
    }

    Ok(CleanupExecuteResult {
        success_count,
        failed,
    })
}

fn file_size(conn: &rusqlite::Connection, path: &str) -> Result<u64, AppError> {
    conn.query_row(
        "SELECT CASE WHEN is_dir = 1 THEN folder_size ELSE size_bytes END
         FROM file_entry WHERE path = ?1",
        [path],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value as u64)
    .map_err(|error| {
        err(
            ErrorCode::InternalError,
            format!("database error: {error}"),
        )
        .with_target("cleanup")
    })
}

fn remove_from_index(conn: &rusqlite::Connection, path: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM file_entry WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'",
        rusqlite::params![path, format!("{}\\%", path)],
    )
    .map_err(|error| {
        err(
            ErrorCode::InternalError,
            format!("database error: {error}"),
        )
        .with_target("cleanup")
    })?;
    Ok(())
}

pub fn paths_from_items(items: &[CleanupExecuteItem]) -> Vec<String> {
    items.iter().map(|item| item.path.clone()).collect()
}
