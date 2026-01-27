// libs/keyforge-protocol/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # `KeyForge` Protocol
//!
//! The Wire Contract for the `KeyForge` system.

#![warn(missing_docs)]

/// Asset-related Data Transfer Objects (DTOs).
pub mod assets;
/// Configuration-related DTOs.
pub mod config;
pub(crate) mod error;
/// Job-related DTOs and messaging.
pub mod job;
/// Node-related DTOs and messaging.
pub mod node;
/// Telemetry-related DTOs.
pub mod telemetry;
/// Primitive and shared protocol types.
pub mod types;

pub mod constants;
pub mod serde_utils;

pub use error::{ErrorCode, ForgeErrorDto};
pub use keyforge_model as model;

// Re-export core DTOs for backward compatibility and convenience
pub use assets::{
    AnalysisReportDto, AssetManifestEntry, BiometricSample, DerivedStatsDto, KeyNodeDto,
    KeyboardGeometryDto, LayoutDto, MetricIdDto, MetricViolationDto, PopulationResponse,
    SwapSuggestionDto, UserStatsStore, ValidationResultDto,
};
pub use config::{
    ConfigAggregateDto, CorpusSourceDto, CostMatrixSourceDto, KeyConstraintDto,
    KeyboardDefinitionDto, ScoringWeightsDto, SearchParamsDto,
};
pub use job::{
    JobConfig, JobDetailedStatus, JobQueueResponse, JobRequest, JobResponse, ResultSubmission,
};
pub use node::{NodeRequest, NodeResponse, NodeTelemetry, TuningProfile};
pub use telemetry::SystemMetrics;
pub use types::{JobIdentifierDto, JobStatusDto, LimitedVec};

/// The current protocol version.
pub const PROTOCOL_VERSION: u32 = 2;
/// Minimum client version supported by this server.
pub const MIN_CLIENT_VERSION: u32 = 1;
/// Minimum server version supported by this client.
pub const MIN_SERVER_VERSION: u32 = 1;

/// Checks if the client and server versions are compatible.
///
/// # Errors
///
/// Returns a string error if the versions are incompatible.
pub fn check_version_compatibility(
    client_version: u32,
    server_version: u32,
    min_client: u32,
    min_server: u32,
) -> Result<(), String> {
    if client_version < min_client {
        return Err(format!(
            "Client version {client_version} is too old (min required: {min_client})"
        ));
    }
    if server_version < min_server {
        return Err(format!(
            "Server version {server_version} is too old (min required: {min_server})"
        ));
    }
    Ok(())
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        assert!(check_version_compatibility(PROTOCOL_VERSION, PROTOCOL_VERSION, 1, 1).is_ok());
        assert!(check_version_compatibility(0, PROTOCOL_VERSION, 1, 1).is_err());
        assert!(check_version_compatibility(PROTOCOL_VERSION, 0, 1, 1).is_err());
    }
}
