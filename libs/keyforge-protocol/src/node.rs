// libs/keyforge-protocol/src/node.rs

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

use crate::PROTOCOL_VERSION;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// Registration request from a worker node to the Hive.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeRequest {
    /// Protocol version.
    pub version: u32,
    /// Unique identifier for the node.
    pub node_id: String,
    /// Human-readable hostname of the node.
    pub hostname: String,
    /// Total number of available CPU cores.
    pub cpu_cores: usize,
    /// Detailed CPU model identifier.
    pub cpu_model: String,
    /// CPU capabilities (e.g., "avx", "sse").
    pub capabilities: Vec<String>,
    /// CPU core count (Deprecated, use `cpu_cores`).
    pub cores: i32,
    /// L2 Cache size in KB.
    pub l2_cache_kb: Option<i32>,
    /// Calibrated operations per second (Physics IPS).
    pub ops_per_sec: f32,
    /// Optional Ed25519 public key for identity verification.
    pub public_key: Option<String>,
}

impl Default for NodeRequest {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            node_id: String::new(),
            hostname: String::new(),
            cpu_cores: 0,
            cpu_model: String::new(),
            capabilities: vec![],
            cores: 0,
            l2_cache_kb: None,
            ops_per_sec: 0.0,
            public_key: None,
        }
    }
}

/// Response from the Hive confirming node registration.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeResponse {
    /// Whether the registration was accepted.
    pub accepted: bool,
    /// Error message if rejected.
    pub secret: Option<String>,
    /// Status message (e.g., "registered", "rejected").
    pub status: String,
    /// Suggested hardware-specific tuning.
    pub tuning: Option<TuningProfile>,
    /// Session token for subsequent requests.
    pub token: Option<String>,
}

/// Periodic telemetry sent by worker nodes.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeTelemetry {
    /// Current Job ID being processed.
    pub job_id: Option<String>,
    /// Current operations per second.
    pub ips: f32,
    /// Node temperature (0.0 to 1.0 normalized or absolute Celsius).
    pub temp: f32,
    /// Best score found in the current batch.
    pub current_best: Option<f32>,
    /// Memory usage metric (e.g., "512 MB").
    pub memory_usage: String,
    /// Total memory used in bytes.
    pub memory_bytes: u64,
    /// Number of active processing threads.
    pub active_threads: usize,
    /// Current CPU usage percentage (0.0 to 100.0).
    pub cpu_usage: f32,
    /// Timestamp of the telemetry report.
    pub timestamp: u64,
}

/// Hardware-specific tuning parameters for worker nodes.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct TuningProfile {
    /// Target physics operations per second.
    pub target_ips: f32,
    /// Preferred number of worker threads.
    pub preferred_threads: usize,
    /// Execution strategy (e.g., "table", "fly").
    pub strategy: String,
    /// Ideal batch size for result submissions.
    pub batch_size: usize,
    /// Total threads to spawn.
    pub thread_count: usize,
}
