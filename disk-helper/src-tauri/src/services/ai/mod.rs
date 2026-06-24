pub mod cloud_deepseek;
pub mod local_ollama;
pub mod service;

pub use service::{AiService, ContextItem};

use crate::error::AppError;

pub trait AiProvider: Send + Sync {
    fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<String, AppError>;
}
