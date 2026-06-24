use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::time::Duration;

use crate::error::{err, AppError, ErrorCode};
use crate::services::ai::AiProvider;

const DEEPSEEK_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const MODEL: &str = "deepseek-chat";
const TIMEOUT_SECS: u64 = 30;

pub struct CloudDeepSeekProvider {
    api_key: String,
    client: Client,
}

impl CloudDeepSeekProvider {
    pub fn new(api_key: String) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .build()
            .map_err(|e| err(ErrorCode::InternalError, format!("HTTP client error: {e}")))?;
        Ok(Self { api_key, client })
    }
}

impl AiProvider for CloudDeepSeekProvider {
    fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AppError> {
        let body = serde_json::json!({
            "model": MODEL,
            "temperature": 0.3,
            "max_tokens": 2048,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt },
            ]
        });

        let response = self
            .client
            .post(DEEPSEEK_URL)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    err(ErrorCode::AiTimeout, "DeepSeek 请求超时")
                } else {
                    err(ErrorCode::AiNetworkError, format!("DeepSeek 网络错误: {e}"))
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
                format!("DeepSeek 响应解析失败: {e}"),
            )
        })?;
        return payload
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| err(ErrorCode::AiProviderError, "DeepSeek 响应缺少 content"));
    }

    let message = response
        .text()
        .unwrap_or_else(|_| status.to_string())
        .chars()
        .take(200)
        .collect::<String>();

    match status {
        StatusCode::UNAUTHORIZED => Err(err(ErrorCode::AiInvalidKey, "DeepSeek API Key 无效")),
        StatusCode::TOO_MANY_REQUESTS => Err(err(ErrorCode::AiRateLimit, "DeepSeek 请求过于频繁")),
        _ if status.is_server_error() => Err(err(
            ErrorCode::AiProviderError,
            format!("DeepSeek 服务异常 ({status}): {message}"),
        )),
        _ => Err(err(
            ErrorCode::AiProviderError,
            format!("DeepSeek 请求失败 ({status}): {message}"),
        )),
    }
}
