// apps/keyforge-cli/src/error.rs

use keyforge_adapter::AdapterError;
use keyforge_evolution::EvolutionError;
use keyforge_infra::error::InfraError;
use keyforge_physics::PhysicsError;
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
    Json(#[from] serde_json::error::Error),
    #[error("Adapter error: {0}")]
    Adapter(#[from] AdapterError),
    #[error("Physics error: {0}")]
    Physics(#[from] PhysicsError),
    #[error("Evolution error: {0}")]
    Evolution(#[from] EvolutionError),
    #[error("Infrastructure error: {0}")]
    Infra(#[from] InfraError),
    #[error("{0}")]
    Other(String),
}

pub type CliResult<T> = std::result::Result<T, CliError>;

impl From<Box<dyn std::error::Error>> for CliError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        CliError::Other(e.to_string())
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::Other(e.to_string())
    }
}
