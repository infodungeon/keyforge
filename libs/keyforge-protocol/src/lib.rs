// libs/keyforge-protocol/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
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
//! The Wire Contract for the `KeyForge` system. This crate defines the Data Transfer Objects (DTOs)
//! used for communication between the Client, Server (Hive), and Worker (Agent).
//!
//! ## Responsibilities
//!
//! * **Serialization:** Defines JSON structure for all API requests/responses.
//! * **Versioning:** Enforces protocol compatibility via `PROTOCOL_VERSION`.
//! * **Validation:** Implements `Validator` for DTOs to ensure data integrity before processing.
//! * **Interop:** Generates TypeScript definitions via `ts-rs`.

#![warn(missing_docs)]

/// Asset management DTOs (Manifests, Samples).
pub mod assets;
pub(crate) mod error;
/// Job management DTOs (Config, Request, Response).
pub mod job;
/// Worker node orchestration DTOs (Heartbeat, Handshake).
pub mod node;
/// System health and performance metrics DTOs.
pub mod telemetry;

pub mod constants;
pub mod serde_utils;

pub use error::{ErrorCode, ErrorResponse};
pub use keyforge_model as model;

// Re-export EVERYTHING to maintain backward compatibility with crate public API
pub use assets::{AssetManifestEntry, BiometricSample, PopulationResponse, UserStatsStore};
pub use job::{
    JobConfig, JobDetailedStatus, JobQueueResponse, JobRequest, JobResponse, ResultSubmission,
};
pub use node::{NodeRequest, NodeResponse, NodeTelemetry, TuningProfile};
pub use telemetry::SystemMetrics;

/// The current protocol version. Incremented on breaking changes.
pub const PROTOCOL_VERSION: u32 = 2;
/// Minimum client version supported by this server.
pub const MIN_CLIENT_VERSION: u32 = 1;
/// Minimum server version supported by this client.
pub const MIN_SERVER_VERSION: u32 = 1;

/// Checks if the client and server versions are compatible.
///
/// # Errors
/// Returns an error message if the client or server versions are below the minimum supported thresholds.
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
    use crate::error::ErrorCode;

    #[test]
    fn test_version_compatibility() {
        assert!(check_version_compatibility(PROTOCOL_VERSION, PROTOCOL_VERSION, 1, 1).is_ok());
        assert!(check_version_compatibility(0, PROTOCOL_VERSION, 1, 1).is_err());
        assert!(check_version_compatibility(PROTOCOL_VERSION, 0, 1, 1).is_err());
    }

    #[test]
    fn test_transport_security_policy() {
        #[derive(serde::Deserialize, Debug)]
        struct Wrapper {
            #[serde(deserialize_with = "crate::serde_utils::deserialize_limited_vec")]
            items: Vec<String>,
        }

        let malicious_json = format!(
            "{{ \"items\": [{}] }}",
            (0..100_001).map(|_| "\"x\"").collect::<Vec<_>>().join(",")
        );
        let result: Result<Wrapper, _> = serde_json::from_str(&malicious_json);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds transport limit"));

        let good_json = "{ \"items\": [\"a\", \"b\"] }";
        let good_result: Result<Wrapper, _> = serde_json::from_str(good_json);
        assert!(good_result.is_ok());
    }
}
