use thiserror::Error;

/// CLI-specific error types with consistent formatting
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Update failed: {0}")]
    Update(String),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CliError>;

impl From<Box<dyn std::error::Error>> for CliError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        CliError::Other(e.to_string())
    }
}
