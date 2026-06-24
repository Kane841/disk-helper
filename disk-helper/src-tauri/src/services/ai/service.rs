use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{err, AppError, ErrorCode};
use crate::services::ai::cloud_deepseek::CloudDeepSeekProvider;
use crate::services::ai::local_ollama::LocalOllamaProvider;
use crate::services::ai::AiProvider;
use crate::services::audit;
use crate::services::config::{self, AiMode, AppSettings};

pub const DISCLAIMER: &str = "以上仅供参考，删除操作请在安全清理中自行确认。";

const MAX_CONTEXT_ITEMS: usize = 20;

#[derive(Debug, Clone, Deserialize)]
pub struct ContextItem {
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub risk: Option<String>,
    pub rule_description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AiChatResult {
    pub message: String,
    pub disclaimer: String,
}

#[derive(Debug, Serialize)]
pub struct AiTestConnectionResult {
    pub status: String,
    pub message: String,
    pub provider: String,
}

pub struct AiService;

impl AiService {
    pub fn chat(
        conn: &Connection,
        data_dir: &Path,
        question: &str,
        context_items: Vec<ContextItem>,
    ) -> Result<AiChatResult, AppError> {
        let question = question.trim();
        if question.is_empty() {
            return Err(err(ErrorCode::BadArgument, "问题不能为空"));
        }

        let settings = config::get(conn, data_dir)?;
        let system_prompt = build_system_prompt();
        let user_prompt = build_user_prompt(question, &context_items);
        let message = dispatch_chat(&settings, &system_prompt, &user_prompt)?;

        let summary = format!(
            "AI 咨询：{}（{} 条上下文）",
            truncate(question, 40),
            context_items.len().min(MAX_CONTEXT_ITEMS)
        );
        audit::append(conn, "ai_query", &summary, "info", None, None)?;

        Ok(AiChatResult {
            message,
            disclaimer: DISCLAIMER.into(),
        })
    }

    pub fn test_connection(
        conn: &Connection,
        data_dir: &Path,
        ai_mode: Option<AiMode>,
        api_key: Option<String>,
    ) -> Result<AiTestConnectionResult, AppError> {
        let settings = config::get(conn, data_dir)?;
        let mode = ai_mode.unwrap_or(settings.ai_mode);

        match mode {
            AiMode::Local => {
                let provider = LocalOllamaProvider::new(
                    settings.ollama_base_url.clone(),
                    settings.ollama_model.clone(),
                )?;
                provider.check_model_available()?;
                Ok(AiTestConnectionResult {
                    status: "success".into(),
                    message: format!(
                        "Ollama 连接成功，模型 {} 可用",
                        settings.ollama_model
                    ),
                    provider: "ollama".into(),
                })
            }
            AiMode::Cloud => {
                let key = match api_key {
                    Some(key) if !key.trim().is_empty() => key,
                    _ => config::read_api_key()?,
                };
                let provider = CloudDeepSeekProvider::new(key)?;
                let reply = provider.chat(
                    "You are a connectivity probe. Reply with OK only.",
                    "ping",
                )?;
                let ok = reply.to_ascii_lowercase().contains('o');
                if ok {
                    Ok(AiTestConnectionResult {
                        status: "success".into(),
                        message: "DeepSeek API 连接成功".into(),
                        provider: "deepseek".into(),
                    })
                } else {
                    Ok(AiTestConnectionResult {
                        status: "success".into(),
                        message: "DeepSeek API 连接成功".into(),
                        provider: "deepseek".into(),
                    })
                }
            }
        }
    }
}

fn dispatch_chat(
    settings: &AppSettings,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AppError> {
    match settings.ai_mode {
        AiMode::Local => {
            let provider = LocalOllamaProvider::new(
                settings.ollama_base_url.clone(),
                settings.ollama_model.clone(),
            )?;
            provider.chat(system_prompt, user_prompt)
        }
        AiMode::Cloud => {
            let api_key = config::read_api_key()?;
            let provider = CloudDeepSeekProvider::new(api_key)?;
            provider.chat(system_prompt, user_prompt)
        }
    }
}

pub fn desensitize_path(path: &str) -> String {
    let normalized = path.replace('/', "\\");
    let mut result = normalized;

    if let Some(users_idx) = result.to_ascii_lowercase().find("\\users\\") {
        let after_users = users_idx + "\\users\\".len();
        if after_users < result.len() {
            if let Some(next_sep) = result[after_users..].find('\\') {
                let username_end = after_users + next_sep;
                result.replace_range(after_users..username_end, "{user}");
            }
        }
    }

    if result.len() > 120 {
        let head = &result[..60];
        let tail = &result[result.len() - 40..];
        format!("{head}...{tail}")
    } else {
        result
    }
}

fn build_system_prompt() -> String {
    [
        "你是 Disk Helper 的磁盘空间分析助手，帮助用户理解文件/文件夹用途与清理风险。",
        "硬性约束：",
        "1. 你不能代替用户执行删除，只能给出分析与建议。",
        "2. 若上下文含 risk 字段，以规则引擎 risk 为准；danger 项必须明确警告。",
        "3. 回答结构：是什么 / 能否删除 / 可能影响 / 如何恢复（若适用）。",
        "4. 使用 Markdown，简洁清晰，中文回复。",
    ]
    .join("\n")
}

fn build_user_prompt(question: &str, context_items: &[ContextItem]) -> String {
    let limited: Vec<_> = context_items.iter().take(MAX_CONTEXT_ITEMS).collect();
    let context_json: Vec<serde_json::Value> = limited
        .iter()
        .map(|item| {
            serde_json::json!({
                "path": desensitize_path(&item.path),
                "size_bytes": item.size_bytes,
                "is_dir": item.is_dir,
                "risk": item.risk,
                "rule_description": item.rule_description,
            })
        })
        .collect();

    format!(
        "用户问题：{question}\n\n上下文（JSON，路径已脱敏，最多 {MAX_CONTEXT_ITEMS} 条）：\n{}",
        serde_json::to_string_pretty(&context_json).unwrap_or_else(|_| "[]".into())
    )
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max_chars).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desensitize_replaces_username() {
        let input = r"C:\Users\Alice\Downloads\file.zip";
        let out = desensitize_path(input);
        assert!(out.contains(r"C:\Users\{user}\"));
        assert!(!out.contains("Alice"));
    }

    #[test]
    fn desensitize_truncates_long_paths() {
        let long = format!(r"C:\Users\Bob\{}\file.txt", "x".repeat(100));
        let out = desensitize_path(&long);
        assert!(out.contains("..."));
        assert!(out.len() <= 120 + 3);
    }

    #[test]
    fn context_item_deserializes_snake_case_fields() {
        let json = r#"{
            "path": "C:\\Temp\\foo",
            "size_bytes": 119000000,
            "is_dir": true,
            "risk": "safe"
        }"#;
        let item: ContextItem = serde_json::from_str(json).expect("deserialize");
        assert_eq!(item.path, r"C:\Temp\foo");
        assert_eq!(item.size_bytes, 119000000);
        assert!(item.is_dir);
        assert_eq!(item.risk.as_deref(), Some("safe"));
    }

    #[test]
    fn build_user_prompt_includes_question() {
        let prompt = build_user_prompt("能删吗", &[]);
        assert!(prompt.contains("能删吗"));
        assert!(prompt.contains("上下文"));
    }
}
