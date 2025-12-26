use thiserror::Error;

#[derive(Error, Debug)]
pub enum InfraError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Hash Mismatch: Expected {expected}, Got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Lock Error: {0}")]
    LockError(String),

    #[error("Config Error: {0}")]
    Config(String),
}

pub type InfraResult<T> = Result<T, InfraError>;
