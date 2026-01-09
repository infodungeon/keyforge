// apps/keyforge-agent/src/models.rs

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


use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

// Re-export specific types if needed
pub use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile};

/// Configuration for the hardware calibration process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationConfig {
    /// Number of keys to simulate during calibration.
    pub key_count: usize,
    /// Number of iterations to run before measuring to warm up the CPU/Cache.
    pub warmup_iterations: usize,
    /// Duration in milliseconds to run the calibration loop.
    pub duration_ms: u64,
    /// Number of scoring operations per batch.
    pub batch_size: usize,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            key_count: 30,
            warmup_iterations: 100,
            duration_ms: 1000,
            batch_size: 100,
        }
    }
}

/// Configuration for network communication and resilience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Request timeout in seconds.
    pub timeout_seconds: u64,
    /// Interval between heartbeat messages in seconds.
    pub heartbeat_interval_seconds: u64,
    /// Maximum backoff duration for retries in seconds.
    pub max_backoff_seconds: u64,
    /// Number of failures before tripping the circuit breaker.
    pub circuit_breaker_threshold: u32,
    /// Cooldown period in seconds after the circuit breaker trips.
    pub circuit_breaker_cooldown: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            heartbeat_interval_seconds: 15,
            max_backoff_seconds: 60,
            circuit_breaker_threshold: 5,
            circuit_breaker_cooldown: 60,
        }
    }
}

/// Configuration for local data maintenance and garbage collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    /// Time-to-live for cached assets in days.
    pub ttl_days: u64,
    /// Interval between pruning checks in seconds.
    pub prune_interval_seconds: u64,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            ttl_days: 7,
            prune_interval_seconds: 3600,
        }
    }
}

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
    
    /// Calibration settings.
    #[serde(default)]
    pub calibration: CalibrationConfig,
    /// Network settings.
    #[serde(default)]
    pub network: NetworkConfig,
    /// Maintenance settings.
    #[serde(default)]
    pub maintenance: MaintenanceConfig,
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
            calibration: CalibrationConfig::default(),
            network: NetworkConfig::default(),
            maintenance: MaintenanceConfig::default(),
        }
    }
}

/// Partial configuration for loading from files (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAgentConfig {
    /// Optional override for Hive URL.
    pub hive_url: Option<String>,
    /// Optional override for Node ID.
    pub node_id: Option<String>,
    /// Optional override for Secret.
    pub secret: Option<String>,
    /// Optional override for Private Key.
    pub private_key: Option<String>,
    /// Optional override for Data Directory.
    pub data_dir: Option<PathBuf>,
    /// Optional override for Core count.
    pub cores: Option<usize>,
    /// Optional override for Calibration settings.
    pub calibration: Option<CalibrationConfig>,
    /// Optional override for Network settings.
    pub network: Option<NetworkConfig>,
    /// Optional override for Maintenance settings.
    pub maintenance: Option<MaintenanceConfig>,
}

impl PartialAgentConfig {
    /// Loads a partial configuration from a file (JSON or TOML).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        if path.as_ref().extension().and_then(|s| s.to_str()) == Some("toml") {
             toml::from_str(&content).map_err(|e| format!("Failed to parse TOML config: {}", e))
        } else {
             serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON config: {}", e))
        }
    }
}

impl AgentConfig {
    /// Merges a partial configuration into this one, overriding existing values.
    pub fn merge(&mut self, partial: PartialAgentConfig) {
        if let Some(v) = partial.hive_url { self.hive_url = v; }
        if let Some(v) = partial.node_id { self.node_id = v; }
        if let Some(v) = partial.secret { self.secret = v; }
        if let Some(v) = partial.private_key { self.private_key = v; }
        if let Some(v) = partial.data_dir { self.data_dir = v; }
        if let Some(v) = partial.cores { self.cores = v; }
        if let Some(v) = partial.calibration { self.calibration = v; }
        if let Some(v) = partial.network { self.network = v; }
        if let Some(v) = partial.maintenance { self.maintenance = v; }
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

    /// Returns a point-in-time snapshot of the metrics as .
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
