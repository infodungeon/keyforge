// ===== keyforge/ui/src-tauri/src/error.rs =====
use keyforge_protocol::ErrorCode;
use serde::Serialize;
use thiserror::Error;

/// Unified error type for Tauri commands in the KeyForge UI.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Errors occurring during filesystem operations.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors occurring during JSON serialization or deserialization.
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Errors related to invalid application configuration.
    #[error("Configuration Error: {0}")]
    Config(String),

    /// Input validation failures for user-provided data.
    #[error("Validation Error: {0}")]
    Validation(String),

    /// Errors encountered during communication with the remote Hive server.
    #[error("Network Error: {0}")]
    Network(String),

    /// Categorized internal logic failures.
    #[error("Internal Error: {0}")]
    Internal(String),
    
    /// Error indicating that a requested resource was not found.
    #[error("Not Found")]
    NotFound, // ADDED
}

/// Standardized error response sent to the frontend for failed commands.
#[derive(Serialize, Debug)]
pub struct CommandErrorResponse {
    /// A stable machine-readable error code.
    pub code: ErrorCode,
    /// A human-readable description of the error.
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
