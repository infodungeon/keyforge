use keyforge_model::error::ForgeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Loader Error: {0}")]
    Loader(#[from] ForgeError),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Config Error: {0}")]
    Config(String),

    #[error("Internal Error: {0}")]
    Internal(String),
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;
