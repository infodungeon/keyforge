pub mod error;
pub mod protocol;

// Re-export Domain Modules from Model to maintain API compatibility
pub use keyforge_model::config;
pub use keyforge_model::constants;
pub use keyforge_model::geometry;
pub use keyforge_model::job;
pub use keyforge_model::keycodes;
pub use keyforge_model::parsing;
pub use keyforge_model::types;
pub use keyforge_model::validator;

// Re-export items at root for convenience (optional, but good for some patterns)
pub use keyforge_model::validator::{Validator, LayoutValidator};

pub use protocol::{
    BiometricSample, JobConfig, JobQueueResponse, JobRequest, JobResponse,
    JobStatus, NodeRequest, NodeResponse, PopulationResponse, ResultSubmission,
    SystemMetrics, TuningProfile, UserStatsStore,
    // These are now re-exported in protocol.rs
    CostMatrixSource, KeyConstraint,
};

pub const PROTOCOL_VERSION: u32 = 2;
pub const MIN_CLIENT_VERSION: u32 = 1;
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
