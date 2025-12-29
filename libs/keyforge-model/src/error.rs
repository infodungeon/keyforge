use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Physics Violation: {0}")]
    Physics(#[from] PhysicsError),

    #[error("Evolution Error: {0}")]
    Evolution(String),

    #[error("Persistence Error: {0}")]
    Persistence(String),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Asset Not Found: {0}")]
    NotFound(String),

    #[error("Internal Error: {0}")]
    Internal(String),

    #[error("Invalid Data: {0}")]
    InvalidData(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum PhysicsError {
    #[error("Hand index {0} is invalid (must be 0 or 1)")]
    InvalidHandIndex(u8),

    #[error("Finger index {0} is invalid (must be 0-4)")]
    InvalidFingerIndex(u8),

    #[error("Matrix dimension mismatch: expected {expected}, found {found}")]
    DimensionMismatch { expected: usize, found: usize },
    
    #[error("Layout size {0} exceeds physical key count {1}")]
    LayoutOverflow(usize, usize),

    #[error("Layout size {0} is insufficient for physical key count {1}")]
    LayoutUnderflow(usize, usize),
}
