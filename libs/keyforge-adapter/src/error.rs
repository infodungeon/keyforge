use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AdapterError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Unknown key token: {0}")]
    UnknownToken(String),

    #[error("Layout string exceeds maximum length of {0}")]
    LayoutTooLong(usize),
}

pub type AdapterResult<T> = Result<T, AdapterError>;
