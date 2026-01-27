// ===== keyforge/ui/src-tauri/src/error.rs =====
use keyforge_infra::error::InfraError;
use keyforge_protocol::ErrorCode;
use serde::Serialize;
use thiserror::Error;

/// Unified error type for Tauri commands in the `KeyForge` UI.
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
    NotFound,
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

impl From<std::string::FromUtf8Error> for CommandError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        CommandError::Internal(e.to_string())
    }
}

impl From<keyforge_infra::error::InfraError> for CommandError {
    fn from(e: keyforge_infra::error::InfraError) -> Self {
        match e {
            InfraError::Io(io) => CommandError::Io(io),
            InfraError::Network(ne) => CommandError::Network(ne.to_string()),
            InfraError::NetworkString(s) => CommandError::Network(s),
            InfraError::Serde(se) => CommandError::Serde(se),
            InfraError::Toml(te) => CommandError::Internal(te.to_string()),
            InfraError::HashMismatch { expected, actual } => CommandError::Validation(format!(
                "Hash mismatch: expected {expected}, got {actual}"
            )),
            InfraError::LockError(s) | InfraError::Config(s) => CommandError::Config(s),
            InfraError::Validation(s) => CommandError::Validation(s),
        }
    }
}

impl From<keyforge_model::error::ForgeError> for CommandError {
    fn from(e: keyforge_model::error::ForgeError) -> Self {
        use keyforge_model::error::ForgeError;
        match e {
            ForgeError::Io(io) => CommandError::Io(io),
            ForgeError::Serde(se) => CommandError::Serde(se),
            ForgeError::Physics(pe) => CommandError::Validation(pe.to_string()),
            ForgeError::Validation(s) | ForgeError::InvalidData(s) => CommandError::Validation(s),
            ForgeError::NotFound(_) => CommandError::NotFound,
            ForgeError::Config(s) => CommandError::Config(s),
            _ => CommandError::Internal(e.to_string()),
        }
    }
}

impl From<keyforge_evolution::EvolutionError> for CommandError {
    fn from(e: keyforge_evolution::EvolutionError) -> Self {
        match e {
            keyforge_evolution::EvolutionError::Physics(pe) => {
                CommandError::Validation(pe.to_string())
            }
            keyforge_evolution::EvolutionError::Config(s) => CommandError::Config(s),
            keyforge_evolution::EvolutionError::Aborted => CommandError::Internal("Aborted".into()),
            keyforge_evolution::EvolutionError::Internal(s) => CommandError::Internal(s),
        }
    }
}

impl From<keyforge_persistence::error::PersistenceError> for CommandError {
    fn from(e: keyforge_persistence::error::PersistenceError) -> Self {
        use keyforge_persistence::error::PersistenceError;
        match e {
            PersistenceError::Io(io) => CommandError::Io(io),
            PersistenceError::Serde(se) => CommandError::Serde(se),
            PersistenceError::ProjectNotFound(_) => CommandError::NotFound,
            PersistenceError::AssetLoad(s) => CommandError::Config(s),
            PersistenceError::Validation(s) => CommandError::Validation(s),
            PersistenceError::Forge(fe) => CommandError::from(fe),
            _ => CommandError::Internal(e.to_string()),
        }
    }
}

impl From<keyforge_adapter::error::AdapterError> for CommandError {
    fn from(e: keyforge_adapter::error::AdapterError) -> Self {
        use keyforge_adapter::error::AdapterError;
        match e {
            AdapterError::Validation(s) | AdapterError::UnknownToken(s) => {
                CommandError::Validation(s)
            }
            AdapterError::LayoutTooLong(n) => {
                CommandError::Validation(format!("Layout too long: {n}"))
            }
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for CommandError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        CommandError::Internal(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for CommandError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        CommandError::Internal(e.to_string())
    }
}

impl From<tauri_plugin_shell::Error> for CommandError {
    fn from(e: tauri_plugin_shell::Error) -> Self {
        CommandError::Internal(e.to_string())
    }
}

impl From<keyforge_physics::PhysicsError> for CommandError {
    fn from(e: keyforge_physics::PhysicsError) -> Self {
        CommandError::Validation(e.to_string())
    }
}
