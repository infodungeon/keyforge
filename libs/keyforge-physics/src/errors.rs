use thiserror::Error;

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
