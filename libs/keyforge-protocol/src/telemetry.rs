// libs/keyforge-protocol/src/telemetry.rs

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

use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// System-wide metrics.
#[derive(Serialize, Deserialize, Debug, Default, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SystemMetrics {
    /// Uptime in seconds.
    pub uptime_secs: u64,
    /// Number of active jobs.
    pub active_jobs: i64,
    /// Total results processed.
    pub total_results: i64,
    /// Number of nodes online.
    pub nodes_online: i64,
    /// Total operations per second across the cluster.
    pub total_ops_per_sec: f32,
    /// Server memory used in bytes.
    pub server_memory_used: u64,
    /// Server CPU usage percentage.
    pub server_cpu_usage: f32,
}
