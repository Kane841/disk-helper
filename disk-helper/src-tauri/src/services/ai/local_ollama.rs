use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::time::Duration;

use crate::error::{err, AppError, ErrorCode};
use crate::services::ai::AiProvider;

const TIMEOUT_SECS: u64 = 120;

pub struct LocalOllamaProvider {
    base_url: String,
    model: String,
    client: Client,
}

impl LocalOllamaProvider {
    pub fn new(base_url: String, model: String) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .build()
            .map_err(|e| err(ErrorCode::InternalError, format!("HTTP client error: {e}")))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            client,
        })
    }

    pub fn check_model_available(&self) -> Result<(), AppError> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().map_err(|e| {
            if e.is_timeout() {
                err(ErrorCode::AiTimeout, "Ollama 连接超时")
            } else {
                err(
                    ErrorCode::AiOllamaUnavailable,
                    "Ollama 未启动或地址不可达，请确认服务已运行",
                )
            }
        })?;

        if !response.status().is_success() {
            return Err(err(
                ErrorCode::AiOllamaUnavailable,
                format!("Ollama 健康检查失败 ({})", response.status()),
            ));
        }

        let payload: serde_json::Value = response.json().map_err(|e| {
            err(
                ErrorCode::AiOllamaUnavailable,
                format!("Ollama 响应解析失败: {e}"),
            )
        })?;

        let models = payload
            .pointer("/models")
            .and_then(|v| v.as_array())
            .ok_or_else(|| err(ErrorCode::AiOllamaUnavailable, "Ollama 响应格式异常"))?;

        let target = self.model.as_str();
        let found = models.iter().any(|entry| {
            entry
                .get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|name| model_matches(name, target))
        });

        if found {
            Ok(())
        } else {
            Err(err(
                ErrorCode::AiModelNotFound,
                format!("未找到模型 {target}，请执行 ollama pull {target}"),
            ))
        }
    }
}

fn model_matches(installed: &str, target: &str) -> bool {
    installed == target
        || installed.starts_with(&format!("{target}:"))
        || target.starts_with(installed)
}

impl AiProvider for LocalOllamaProvider {
    fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AppError> {
        self.check_model_available()?;

        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "temperature": 0.3,
            "max_tokens": 2048,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt },
            ]
        });

        let response = self.client.post(url).json(&body).send().map_err(|e| {
            if e.is_timeout() {
                err(ErrorCode::AiTimeout, "Ollama 请求超时")
            } else {
                err(
                    ErrorCode::AiOllamaUnavailable,
                    format!("Ollama 网络错误: {e}"),
                )
            }
        })?;

        map_response(response)
    }
}

fn map_response(response: reqwest::blocking::Response) -> Result<String, AppError> {
    let status = response.status();
    if status.is_success() {
        let payload: serde_json::Value = response.json().map_err(|e| {
            err(
                ErrorCode::AiProviderError,
                format!("Ollama 响应解析失败: {e}"),
            )
        })?;
        return payload
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| err(ErrorCode::AiProviderError, "Ollama 响应缺少 content"));
    }

    let message = response
        .text()
        .unwrap_or_else(|_| status.to_string())
        .chars()
        .take(200)
        .collect::<String>();

    match status {
        StatusCode::NOT_FOUND => Err(err(
            ErrorCode::AiModelNotFound,
            "Ollama 模型未找到，请确认已 pull 目标模型",
        )),
        _ if status.is_server_error() => Err(err(
            ErrorCode::AiProviderError,
            format!("Ollama 服务异常 ({status}): {message}"),
        )),
        _ => Err(err(
            ErrorCode::AiProviderError,
            format!("Ollama 请求失败 ({status}): {message}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_matches_variants() {
        assert!(model_matches("deepseek-r1:1.5b", "deepseek-r1:1.5b"));
        assert!(model_matches("deepseek-r1:1.5b:latest", "deepseek-r1:1.5b"));
        assert!(!model_matches("llama3:8b", "deepseek-r1:1.5b"));
    }
}
