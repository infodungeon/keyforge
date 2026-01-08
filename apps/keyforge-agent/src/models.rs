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
    pub hive_url: String,
    pub node_id: String,
    pub secret: String,
    pub private_key: String,
    pub data_dir: PathBuf,
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
    // f32 bits stored as u32
    pub ips: AtomicU32,
    pub temp: AtomicU32,
    pub best_score: AtomicU32,
    pub job_id_hash: AtomicU64, // Partial hash for identification
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
    pub fn update(&self, ips: f32, temp: f32, best_score: f32) {
        self.ips.store(ips.to_bits(), Ordering::Relaxed);
        self.temp.store(temp.to_bits(), Ordering::Relaxed);
        self.best_score.store(best_score.to_bits(), Ordering::Relaxed);
    }

    pub fn set_job_id(&self, id: &str) {
        if let Ok(mut lock) = self.current_job_id.write() {
            *lock = id.to_string();
        }
    }

    pub fn get_job_id(&self) -> String {
        self.current_job_id.read().map(|s| s.clone()).unwrap_or_else(|_| "unknown".to_string())
    }

    pub fn snapshot(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.ips.load(Ordering::Relaxed)),
            f32::from_bits(self.temp.load(Ordering::Relaxed)),
            f32::from_bits(self.best_score.load(Ordering::Relaxed)),
        )
    }
}

pub type SharedTelemetry = Arc<AgentTelemetry>;
