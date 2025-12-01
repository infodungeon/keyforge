
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyForgeError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV Parsing Error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON Parsing Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("Data Validation Error: {0}")]
    Validation(String),
}

pub type KfResult<T> = Result<T, KeyForgeError>;