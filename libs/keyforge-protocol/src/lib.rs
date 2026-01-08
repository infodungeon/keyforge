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


//! # KeyForge Protocol
//!
//! The Wire Contract for the KeyForge system. This crate defines the Data Transfer Objects (DTOs)
//! used for communication between the Client, Server (Hive), and Worker (Agent).
//!
//! ## Responsibilities
//!
//! * **Serialization:** Defines JSON structure for all API requests/responses.
//! * **Versioning:** Enforces protocol compatibility via `PROTOCOL_VERSION`.
//! * **Validation:** Implements `Validator` for DTOs to ensure data integrity before processing.
//! * **Interop:** Generates TypeScript definitions via `ts-rs`.

#![warn(missing_docs)]

pub(crate) mod error;
pub(crate) mod protocol;

#[cfg(test)]
mod tests;

pub use error::{ErrorCode, ErrorResponse};
pub use keyforge_model as model;

pub use protocol::{
    AssetManifestEntry, BiometricSample, JobConfig, JobQueueResponse, JobRequest, JobResponse,
    JobStatus, NodeRequest, NodeResponse, NodeTelemetry, PopulationResponse, ResultSubmission,
    SystemMetrics, TuningProfile, UserStatsStore,
};

/// The current protocol version. Incremented on breaking changes.
pub const PROTOCOL_VERSION: u32 = 2;
/// Minimum client version supported by this server.
pub const MIN_CLIENT_VERSION: u32 = 1;
/// Minimum server version supported by this client.
pub const MIN_SERVER_VERSION: u32 = 1;

/// Checks if the client and server versions are compatible.
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
