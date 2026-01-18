// apps/keyforge-agent/src/agent/errors.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use thiserror::Error;

/// The primary error type for operations within the `KeyForge` Agent.
#[derive(Error, Debug)]
pub enum AgentError {
    /// Error related to node identity or cryptographic keys.
    #[error("Identity Error: {0}")]
    Identity(String),

    /// Error occurred while detecting or calibrating host hardware.
    #[error("Hardware Detection Error: {0}")]
    Hardware(String),

    /// Error during hardware-specific calibration (e.g., IPS measurement).
    #[error("Calibration Error: {0}")]
    Calibration(String),

    /// Error in network communication (HTTP or WebSocket).
    #[error("Network Error: {0}")]
    Network(String),

    /// Generic internal error or logic violation.
    #[error("Internal Error: {0}")]
    Internal(String),

    /// Error related to system resources (e.g., file descriptors, memory).
    #[error("Resource Error: {0}")]
    Resource(String),
}

/// A specialized Result type for Agent operations.
pub type AgentResult<T> = Result<T, AgentError>;
