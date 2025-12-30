use keyforge_model::error::ForgeError;
use thiserror::Error;

/// Errors that can occur during persistence operations.
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Error propagated from the asset loader.
    #[error("Loader Error: {0}")]
    Loader(#[from] ForgeError),

    /// Standard IO error.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// Error during JSON (de)serialization.
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Configuration or logic error.
    #[error("Config Error: {0}")]
    Config(String),

    /// Internal error, often related to concurrency (e.g., poisoned mutex).
    #[error("Internal Error: {0}")]
    Internal(String),
}

/// A specialized Result type for persistence operations.
pub type PersistenceResult<T> = Result<T, PersistenceError>;
