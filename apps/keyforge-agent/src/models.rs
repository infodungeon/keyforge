// apps/keyforge-agent/src/models.rs

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


use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// Re-export specific types if needed
pub use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile};

/// Configuration specific to the Agent.
/// This resolves the Option types from CommonConfig into concrete values required for runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// The base URL of the Hive server.
    pub hive_url: String,
    /// The unique identifier for this node.
    pub node_id: String,
    /// The secret key for authenticating with the Hive.
    pub secret: String,
    /// The hex-encoded Ed25519 private key for signing results.
    pub private_key: String,
    /// The directory where assets and runtime data are stored.
    pub data_dir: PathBuf,
    /// The number of CPU cores to use for optimization.
    pub cores: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            hive_url: "http://localhost:3000".to_string(),
            node_id: "unknown".to_string(),
            secret: "".to_string(),
            private_key: "".to_string(),
            data_dir: PathBuf::from("data"),
            cores: 1,
        }
    }
}

/// Shared state for real-time telemetry.
/// Uses atomics for lock-free updates from the hot loop.
#[derive(Debug)]
pub struct AgentTelemetry {
    /// Throughput in 'Items Per Second' (standard f32 bits).
    pub ips: AtomicU32,
    /// Estimated core temperature (standard f32 bits).
    pub temp: AtomicU32,
    /// The best (lowest) score found so far for the current job.
    pub best_score: AtomicU32,
    /// A partial hash of the current job ID for quick identification.
    pub job_id_hash: AtomicU64,
    /// The full ID of the job currently being processed.
    pub current_job_id: RwLock<String>,
}

impl Default for AgentTelemetry {
    fn default() -> Self {
        Self {
            ips: AtomicU32::new(0),
            temp: AtomicU32::new(0),
            best_score: AtomicU32::new(0),
            job_id_hash: AtomicU64::new(0),
            current_job_id: RwLock::new("idle".to_string()),
        }
    }
}

impl AgentTelemetry {
    /// Atomic update of the telemetry metrics.
    pub fn update(&self, ips: f32, temp: f32, best_score: f32) {
        self.ips.store(ips.to_bits(), Ordering::Relaxed);
        self.temp.store(temp.to_bits(), Ordering::Relaxed);
        self.best_score.store(best_score.to_bits(), Ordering::Relaxed);
    }

    /// Safely sets the current job ID.
    pub fn set_job_id(&self, id: &str) {
        if let Ok(mut lock) = self.current_job_id.write() {
            *lock = id.to_string();
        }
    }

    /// Returns a copy of the current job ID.
    pub fn get_job_id(&self) -> String {
        self.current_job_id.read().map(|s| s.clone()).unwrap_or_else(|_| "unknown".to_string())
    }

    /// Returns a point-in-time snapshot of the metrics as `(ips, temp, best_score)`.
    pub fn snapshot(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.ips.load(Ordering::Relaxed)),
            f32::from_bits(self.temp.load(Ordering::Relaxed)),
            f32::from_bits(self.best_score.load(Ordering::Relaxed)),
        )
    }
}

/// A thread-safe, reference-counted handle to the agent's telemetry state.
pub type SharedTelemetry = Arc<AgentTelemetry>;
