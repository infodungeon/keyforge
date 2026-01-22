// libs/keyforge-protocol/src/node.rs

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

use crate::PROTOCOL_VERSION;
use keyforge_model::Validator;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

fn default_version() -> u32 {
    PROTOCOL_VERSION
}

/// Real-time status report from a Worker Node (Hot Path).
/// Sent via WebSocket text frames to Hive, then serialized to Valkey.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeTelemetry {
    /// The Job currently being processed.
    pub job_id: Option<String>,
    /// Iterations Per Second (Performance).
    pub ips: f32,
    /// Current Annealing Temperature (State).
    pub temp: f32,
    /// Best score found in this session (Local Best).
    pub current_best: Option<f32>,
    /// Total memory usage in bytes.
    pub memory_usage: u64,
    /// Timestamp of this sample.
    pub timestamp: u64,
}

/// Request from a node to register or heartbeat.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeRequest {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// The Node ID.
    pub node_id: String,
    /// CPU model name.
    pub cpu_model: String,
    /// Number of cores.
    pub cores: i32,
    /// L2 cache size in KB.
    pub l2_cache_kb: Option<i32>,
    /// Operations per second benchmark.
    pub ops_per_sec: f32,
    /// Public key for verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

impl Validator for NodeRequest {
    fn validate(&self) -> Result<(), String> {
        if self.node_id.trim().is_empty() {
            return Err("node_id cannot be empty".into());
        }
        if self.cores <= 0 {
            return Err("cores must be > 0".into());
        }
        if self.ops_per_sec < 0.0 {
            return Err("ops_per_sec cannot be negative".into());
        }
        Ok(())
    }
}

/// Tuning profile for a worker.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct TuningProfile {
    /// Strategy name.
    pub strategy: String,
    /// Batch size for processing.
    pub batch_size: usize,
    /// Number of threads to use.
    pub thread_count: usize,
}

/// Response to a node heartbeat.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct NodeResponse {
    /// Status of the node (e.g., "Active").
    pub status: String,
    /// Tuning profile to apply.
    pub tuning: TuningProfile,
    /// Optional session token (Task-sec-022).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_request_validation() {
        let valid = NodeRequest {
            version: PROTOCOL_VERSION,
            node_id: "node-1".into(),
            cpu_model: "test".into(),
            cores: 8,
            l2_cache_kb: None,
            ops_per_sec: 1000.0,
            public_key: None,
        };
        assert!(valid.validate().is_ok());

        let invalid_id = NodeRequest {
            node_id: " ".into(),
            ..valid.clone()
        };
        assert!(invalid_id.validate().is_err());

        let invalid_cores = NodeRequest {
            cores: 0,
            ..valid.clone()
        };
        assert!(invalid_cores.validate().is_err());

        let invalid_ops = NodeRequest {
            ops_per_sec: -1.0,
            ..valid.clone()
        };
        assert!(invalid_ops.validate().is_err());
    }

    #[test]
    fn test_default_version() {
        assert_eq!(default_version(), PROTOCOL_VERSION);
    }
}
