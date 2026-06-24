use serde::Serialize;

pub mod index;
pub mod cleanup;

#[derive(Debug, Clone, Serialize)]
pub struct VolumeInfo {
    pub drive: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f64,
}
