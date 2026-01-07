use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Identity Error: {0}")]
    Identity(String),

    #[error("Hardware Detection Error: {0}")]
    Hardware(String),

    #[error("Calibration Error: {0}")]
    Calibration(String),

    #[error("Network Error: {0}")]
    Network(String),

    #[error("Internal Error: {0}")]
    Internal(String),

    #[error("Resource Error: {0}")]
    Resource(String),
}

pub type AgentResult<T> = Result<T, AgentError>;
