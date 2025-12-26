// ===== keyforge/ui/src-tauri/src/error.rs =====
use keyforge_protocol::error::ErrorCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Network Error: {0}")]
    Network(String),

    #[error("Internal Error: {0}")]
    Internal(String),

    #[error("Not Found")]
    NotFound, // ADDED
}

#[derive(Serialize)]
pub struct CommandErrorResponse {
    pub code: ErrorCode,
    pub message: String,
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (code, msg) = match self {
            CommandError::Io(e) => (ErrorCode::InternalError, e.to_string()),
            CommandError::Serde(e) => (ErrorCode::BadRequest, e.to_string()),
            CommandError::Config(s) => (ErrorCode::BadRequest, s.clone()),
            CommandError::Validation(s) => (ErrorCode::JobValidationFailed, s.clone()),
            CommandError::Network(s) => (ErrorCode::UpstreamTimeout, s.clone()),
            CommandError::Internal(s) => (ErrorCode::InternalError, s.clone()),
            CommandError::NotFound => (ErrorCode::NotFound, "Resource not found".to_string()),
        };

        CommandErrorResponse { code, message: msg }.serialize(serializer)
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        CommandError::Internal(s)
    }
}

impl From<&str> for CommandError {
    fn from(s: &str) -> Self {
        CommandError::Internal(s.to_string())
    }
}
