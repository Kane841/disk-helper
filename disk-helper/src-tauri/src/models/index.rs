use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub folder_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryStat {
    pub code: String,
    pub size_bytes: u64,
    pub ratio: f64,
}
