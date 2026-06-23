use serde::Serialize;

#[allow(dead_code)] // M2 skeleton — error codes used by later command handlers

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ErrorCode {
    BadArgument,
    NotFound,
    ScanAlreadyRunning,
    IndexNotReady,
    DangerNotConfirmed,
    QuarantineConflict,
    DiskSpaceInsufficient,
    AiNoApiKey,
    AiNetworkError,
    InternalError,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AppError>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: AppError) -> Self {
        Self {
            data: None,
            error: Some(error),
        }
    }
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            target: None,
        }
    }

    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}

pub fn err(code: ErrorCode, message: impl Into<String>) -> AppError {
    AppError::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::VolumeInfo;

    #[test]
    fn ok_response_serializes_data_only() {
        let json = serde_json::to_string(&ApiResponse::ok(VolumeInfo {
            drive: "C:".into(),
            total_bytes: 100,
            used_bytes: 60,
            free_bytes: 40,
            usage_percent: 60.0,
        }))
        .unwrap();
        assert!(json.contains("\"data\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn err_response_serializes_error_only() {
        let json = serde_json::to_string(&ApiResponse::<VolumeInfo>::err(
            err(ErrorCode::BadArgument, "invalid path").with_target("scan"),
        ))
        .unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("BadArgument"));
        assert!(!json.contains("\"data\""));
    }
}
