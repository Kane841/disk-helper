use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CleanupSuggestion {
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub risk: String,
    pub category: String,
    pub rule_id: String,
    pub description: String,
    pub restore_hint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupExecuteItem {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupFailedItem {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuarantineItemDto {
    pub id: String,
    pub original_path: String,
    pub quarantine_path: String,
    pub size_bytes: u64,
    pub moved_at: String,
    pub expires_at: String,
    pub status: String,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditLogDto {
    pub id: String,
    pub occurred_at: String,
    pub event_type: String,
    pub summary: String,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_path: Option<String>,
}
