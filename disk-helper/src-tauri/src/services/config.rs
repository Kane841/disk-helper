use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::{err, AppError, ErrorCode};
use crate::services::audit;

const SETTINGS_KEY: &str = "settings";
const KEYRING_SERVICE: &str = "DiskHelper";
const KEYRING_USER: &str = "ai_api_key";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SoftDeleteTarget {
    Quarantine,
    RecycleBin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AppSettings {
    pub theme: String,
    pub ai_mode: AiMode,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub has_api_key: bool,
    pub quarantine_root: String,
    pub retention_days: u32,
    pub admin_scan_enabled: bool,
    pub warning_threshold_gb: u32,
    pub critical_threshold_gb: u32,
    pub soft_delete_target: SoftDeleteTarget,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigSaveInput {
    pub theme: Option<String>,
    pub ai_mode: Option<AiMode>,
    pub ollama_base_url: Option<String>,
    pub ollama_model: Option<String>,
    pub api_key: Option<String>,
    pub quarantine_root: Option<String>,
    pub retention_days: Option<u32>,
    pub admin_scan_enabled: Option<bool>,
    pub warning_threshold_gb: Option<u32>,
    pub critical_threshold_gb: Option<u32>,
    pub soft_delete_target: Option<SoftDeleteTarget>,
}

impl AppSettings {
    pub fn defaults(data_dir: &Path) -> Self {
        let quarantine_root = data_dir.join("quarantine");
        Self {
            theme: "system".into(),
            ai_mode: AiMode::Local,
            ollama_base_url: "http://127.0.0.1:11434".into(),
            ollama_model: "deepseek-r1:1.5b".into(),
            has_api_key: false,
            quarantine_root: quarantine_root.to_string_lossy().into_owned(),
            retention_days: 30,
            admin_scan_enabled: false,
            warning_threshold_gb: 10,
            critical_threshold_gb: 2,
            soft_delete_target: SoftDeleteTarget::Quarantine,
        }
    }
}

pub fn get(conn: &Connection, data_dir: &Path) -> Result<AppSettings, AppError> {
    let mut settings = AppSettings::defaults(data_dir);
    let stored: Option<String> = conn
        .query_row(
            "SELECT value_json FROM app_settings WHERE key = ?1",
            [SETTINGS_KEY],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = stored {
        let parsed: AppSettings = serde_json::from_str(&json).map_err(|e| {
            err(
                ErrorCode::InternalError,
                format!("invalid settings JSON: {e}"),
            )
        })?;
        settings = parsed;
    }

    settings.has_api_key = read_api_key().is_ok();
    Ok(settings)
}

pub fn save(
    conn: &Connection,
    data_dir: &Path,
    input: ConfigSaveInput,
) -> Result<AppSettings, AppError> {
    let mut settings = get(conn, data_dir)?;
    let previous_ai_mode = settings.ai_mode.clone();

    if let Some(theme) = input.theme {
        settings.theme = theme;
    }
    if let Some(ai_mode) = input.ai_mode {
        settings.ai_mode = ai_mode;
    }
    if let Some(url) = input.ollama_base_url {
        settings.ollama_base_url = url.trim_end_matches('/').to_string();
    }
    if let Some(model) = input.ollama_model {
        settings.ollama_model = model;
    }
    if let Some(root) = input.quarantine_root {
        settings.quarantine_root = root;
    }
    if let Some(days) = input.retention_days {
        settings.retention_days = days.max(1);
    }
    if let Some(enabled) = input.admin_scan_enabled {
        settings.admin_scan_enabled = enabled;
    }
    if let Some(warn) = input.warning_threshold_gb {
        settings.warning_threshold_gb = warn;
    }
    if let Some(crit) = input.critical_threshold_gb {
        settings.critical_threshold_gb = crit;
    }
    if let Some(target) = input.soft_delete_target {
        settings.soft_delete_target = target;
    }

    if let Some(api_key) = input.api_key {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            delete_api_key()?;
            settings.has_api_key = false;
        } else {
            write_api_key(trimmed)?;
            settings.has_api_key = true;
        }
    } else {
        settings.has_api_key = read_api_key().is_ok();
    }

    let json = serde_json::to_string(&settings).map_err(|e| {
        err(
            ErrorCode::InternalError,
            format!("serialize settings failed: {e}"),
        )
    })?;
    let updated_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO app_settings (key, value_json, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
        params![SETTINGS_KEY, json, updated_at],
    )
    .map_err(map_sqlite_err)?;

    if settings.ai_mode != previous_ai_mode {
        let summary = match settings.ai_mode {
            AiMode::Local => "AI 模式切换为本地 Ollama",
            AiMode::Cloud => "AI 模式切换为云端 DeepSeek",
        };
        audit::append(conn, "settings_change", summary, "info", None, None)?;
    }

    Ok(settings)
}

pub fn read_api_key() -> Result<String, AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| {
        err(
            ErrorCode::InternalError,
            format!("keyring entry error: {e}"),
        )
    })?;
    entry.get_password().map_err(|_| {
        err(ErrorCode::AiNoApiKey, "DeepSeek API Key 未配置")
    })
}

fn write_api_key(key: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| {
        err(
            ErrorCode::InternalError,
            format!("keyring entry error: {e}"),
        )
    })?;
    entry.set_password(key).map_err(|e| {
        err(
            ErrorCode::InternalError,
            format!("保存 API Key 失败: {e}"),
        )
    })
}

fn delete_api_key() -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| {
        err(
            ErrorCode::InternalError,
            format!("keyring entry error: {e}"),
        )
    })?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(err(
            ErrorCode::InternalError,
            format!("删除 API Key 失败: {e}"),
        )),
    }
}

fn map_sqlite_err(error: rusqlite::Error) -> AppError {
    err(ErrorCode::InternalError, format!("database error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn defaults_use_data_dir_quarantine() {
        let temp = tempfile::tempdir().expect("temp dir");
        let settings = AppSettings::defaults(temp.path());
        assert!(settings.quarantine_root.contains("quarantine"));
        assert_eq!(settings.ai_mode, AiMode::Local);
        assert_eq!(settings.ollama_model, "deepseek-r1:1.5b");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let temp = tempfile::tempdir().expect("temp dir");
        let conn = db::open(temp.path()).expect("open db");
        let input = ConfigSaveInput {
            ai_mode: Some(AiMode::Cloud),
            ollama_base_url: Some("http://localhost:11434".into()),
            ..Default::default()
        };
        let saved = save(&conn, temp.path(), input).expect("save");
        assert_eq!(saved.ai_mode, AiMode::Cloud);

        let loaded = get(&conn, temp.path()).expect("get");
        assert_eq!(loaded.ai_mode, AiMode::Cloud);
        assert_eq!(loaded.ollama_base_url, "http://localhost:11434");
    }
}
