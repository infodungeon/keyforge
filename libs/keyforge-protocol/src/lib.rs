pub(crate) mod error;
pub(crate) mod protocol;
pub mod parsing;
pub mod keycodes;

pub use error::{ErrorCode, ErrorResponse};

pub mod dtos;

pub use dtos::config;
pub use dtos::constants;
pub use dtos::geometry;
pub use dtos::job;
pub use dtos::kle;
pub use dtos::types; // keycodes merged into types or not used?
// pub use dtos::keycodes; // Not copied, assuming subset in types/constants
pub use dtos::validator;

pub use dtos::validator::{Validator, LayoutValidator};

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
