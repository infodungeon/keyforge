use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyForgeError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Compression Error: {0}")]
    Compression(String),

    #[error("MessagePack Error: {0}")]
    MsgPack(String),

    #[error("Asset Not Found: {0}")]
    NotFound(String),

    #[error("Invalid Data: {0}")]
    InvalidData(String),

    #[error("Internal Error: {0}")]
    Internal(String),
}
