// libs/keyforge-protocol/src/telemetry.rs

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

use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// Global health and performance metrics for the Hive cluster.
#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SystemMetrics {
    /// Total number of nodes registered in the system.
    pub total_nodes: usize,
    /// Total aggregate operations per second across all nodes.
    pub total_ips: f32,
    /// Number of currently active (running) jobs.
    pub active_jobs: usize,
    /// Number of jobs waiting in the queue.
    pub pending_jobs: usize,
    /// Total number of jobs completed since startup.
    pub completed_jobs: usize,
    /// Total number of results processed.
    pub total_results: u64,
    /// System uptime in seconds.
    pub uptime_secs: u64,
    /// Number of nodes currently heartbeating.
    pub nodes_online: usize,
    /// Total operations per second (deprecated, use `total_ips`).
    pub total_ops_per_sec: f32,
    /// Server memory used in bytes.
    pub server_memory_used: u64,
    /// Server CPU usage percentage (0.0 to 100.0).
    pub server_cpu_usage: f32,
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_serde() {
        let metrics = SystemMetrics {
            total_nodes: 10,
            total_ips: 1500.5,
            ..Default::default()
        };

        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: SystemMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert!((metrics.total_ips - deserialized.total_ips).abs() < f32::EPSILON);
    }
}
