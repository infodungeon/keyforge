// apps/keyforge-agent/src/models.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

pub use keyforge_protocol::{NodeRequest, NodeResponse, TuningProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationConfig {
    pub key_count: usize,
    pub warmup_iterations: usize,
    pub duration_ms: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub timeout_seconds: u64,
    pub heartbeat_interval_seconds: u64,
    pub max_backoff_seconds: u64,
    pub initial_backoff_seconds: u64,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_cooldown: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            heartbeat_interval_seconds: 15,
            max_backoff_seconds: 60,
            initial_backoff_seconds: 1,
            circuit_breaker_threshold: 5,
            circuit_breaker_cooldown: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaintenanceConfig {
    pub ttl_days: u64,
    pub prune_interval_seconds: u64,
    pub prune_target_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeConfig {
    pub max_corpora_sources: usize,
    pub job_timeout_sec: u64,
    pub keycodes_file: String,
    pub default_search_seed: u64,
}

impl Default for ComputeConfig {
    fn default() -> Self {
        Self {
            max_corpora_sources: 50,
            job_timeout_sec: 3600,
            keycodes_file: "keycodes".to_string(),
            default_search_seed: 42,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub progress_log_sampling_rate: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            progress_log_sampling_rate: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub default_filter: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_filter: "info,keyforge_agent=debug".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub node_id_prefix: String,
    pub shutdown_channel_capacity: usize,
    pub result_channel_capacity: usize,
    pub config_dir_name: String,
    pub key_file_name: String,
    pub idle_job_id: String,
    pub machine_id_override: Option<String>,
    pub corpora_dir_name: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            node_id_prefix: "agent-".to_string(),
            shutdown_channel_capacity: 16,
            result_channel_capacity: 100,
            config_dir_name: "keyforge".to_string(),
            key_file_name: "agent.key.age".to_string(),
            idle_job_id: "idle".to_string(),
            machine_id_override: None,
            corpora_dir_name: "corpora".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub hive_url: String,
    pub asset_url: String, // NEW field
    pub node_id: String,
    pub secret: String,
    pub private_key: String,
    pub data_dir: PathBuf,
    pub cores: usize,
    #[serde(default)]
    pub calibration: CalibrationConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub maintenance: MaintenanceConfig,
    #[serde(default)]
    pub compute: ComputeConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub system: SystemConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            hive_url: "https://hive.infodungeon.com:3000".to_string(),
            asset_url: "http://localhost:3001".to_string(),
            node_id: "unknown".to_string(),
            secret: String::new(),
            private_key: String::new(),
            data_dir: PathBuf::from("data"),
            cores: 1,
            calibration: CalibrationConfig::default(),
            network: NetworkConfig::default(),
            maintenance: MaintenanceConfig::default(),
            compute: ComputeConfig::default(),
            telemetry: TelemetryConfig::default(),
            logging: LoggingConfig::default(),
            system: SystemConfig::default(),
        }
    }
}

// Partial Config for file loading
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAgentConfig {
    pub hive_url: Option<String>,
    pub asset_url: Option<String>, // NEW field
    pub node_id: Option<String>,
    pub secret: Option<String>,
    pub private_key: Option<String>,
    pub data_dir: Option<PathBuf>,
    pub cores: Option<usize>,
    pub calibration: Option<CalibrationConfig>,
    pub network: Option<NetworkConfig>,
    pub maintenance: Option<MaintenanceConfig>,
    pub compute: Option<ComputeConfig>,
    pub telemetry: Option<TelemetryConfig>,
    pub logging: Option<LoggingConfig>,
    pub system: Option<SystemConfig>,
}

impl PartialAgentConfig {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext == "zst" || path.to_string_lossy().ends_with(".mpk.zst") {
                let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
                let decoder = zstd::Decoder::new(file).map_err(|e| e.to_string())?;
                return rmp_serde::from_read(decoder).map_err(|e| e.to_string());
            }
        }
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content).map_err(|e| e.to_string())
        } else {
            serde_json::from_str(&content).map_err(|e| e.to_string())
        }
    }
}

impl AgentConfig {
    pub fn merge(&mut self, partial: PartialAgentConfig) {
        if let Some(v) = partial.hive_url {
            self.hive_url = v;
        }
        if let Some(v) = partial.asset_url {
            self.asset_url = v;
        }
        if let Some(v) = partial.node_id {
            self.node_id = v;
        }
        if let Some(v) = partial.secret {
            self.secret = v;
        }
        if let Some(v) = partial.private_key {
            self.private_key = v;
        }
        if let Some(v) = partial.data_dir {
            self.data_dir = v;
        }
        if let Some(v) = partial.cores {
            self.cores = v;
        }
        if let Some(v) = partial.calibration {
            self.calibration = v;
        }
        if let Some(v) = partial.network {
            self.network = v;
        }
        if let Some(v) = partial.maintenance {
            self.maintenance = v;
        }
        if let Some(v) = partial.compute {
            self.compute = v;
        }
        if let Some(v) = partial.telemetry {
            self.telemetry = v;
        }
        if let Some(v) = partial.logging {
            self.logging = v;
        }
        if let Some(v) = partial.system {
            self.system = v;
        }
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_config_merging() {
        let mut base = AgentConfig::default();

        let partial = PartialAgentConfig {
            hive_url: Some("http://file-config.com".to_string()),
            cores: Some(8),
            calibration: Some(CalibrationConfig {
                key_count: 50,
                ..Default::default()
            }),
            ..Default::default()
        };

        base.merge(partial);

        assert_eq!(base.hive_url, "http://file-config.com");
        assert_eq!(base.cores, 8);
        assert_eq!(base.calibration.key_count, 50);
        assert_eq!(base.network.timeout_seconds, 30);
    }
}

#[derive(Debug)]
pub struct AgentTelemetry {
    pub ips: AtomicU32,
    pub temp: AtomicU32,
    pub best_score: AtomicU32,
    pub job_id_hash: AtomicU64,
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
        self.best_score
            .store(best_score.to_bits(), Ordering::Relaxed);
    }
    pub fn set_job_id(&self, id: &str) {
        if let Ok(mut lock) = self.current_job_id.write() {
            *lock = id.to_string();
        }
    }
    pub fn get_job_id(&self) -> String {
        self.current_job_id
            .read()
            .map_or_else(|_| "unknown".to_string(), |s| s.clone())
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
