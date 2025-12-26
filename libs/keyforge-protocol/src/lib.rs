pub mod config;
pub mod constants;
pub mod error;
pub mod geometry;
pub mod job;
pub mod protocol;

// Re-export new names for easier access
pub use protocol::{
    BiometricSample, CostMatrixSource, JobConfig, JobQueueResponse, JobRequest, JobResponse,
    JobStatus, KeyConstraint, NodeRequest, NodeResponse, PopulationResponse, ResultSubmission,
    SystemMetrics, TuningProfile, UserStatsStore,
};

// Semantic Versioning for Data Contracts
pub const PROTOCOL_VERSION: u32 = 1;

/// Shared validation trait for data transfer objects.
pub trait Validator {
    fn validate(&self) -> Result<(), String>;
}

pub struct LayoutValidator;
impl LayoutValidator {
    pub fn validate_structure(layout: &str) -> Result<(), String> {
        if layout.trim().is_empty() {
            return Err("Layout is empty".to_string());
        }
        // Basic check: ensure it has enough keys (heuristic)
        if layout.split_whitespace().count() < 10 {
            return Err("Layout has too few keys".to_string());
        }
        Ok(())
    }
}

pub mod keycodes;

/// Minimum version of the client that this server supports.
pub const MIN_CLIENT_VERSION: u32 = 1;

/// Minimum version of the server that this client supports.
pub const MIN_SERVER_VERSION: u32 = 1;

pub fn check_version_compatibility(client_version: u32, server_version: u32) -> Result<(), String> {
    if client_version < MIN_CLIENT_VERSION {
        return Err(format!(
            "Client version {} is too old (min required: {})",
            client_version, MIN_CLIENT_VERSION
        ));
    }
    if server_version < MIN_SERVER_VERSION {
        return Err(format!(
            "Server version {} is too old (min required: {})",
            server_version, MIN_SERVER_VERSION
        ));
    }
    Ok(())
}
pub mod parsing;
