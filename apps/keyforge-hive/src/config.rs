use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConfig {
    #[serde(default)]
    pub queue: QueueConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    
    /// Connection string for the coordination layer.
    /// Defaults to localhost for dev, overridden by env var in Docker.
    #[serde(default = "default_valkey")]
    pub valkey_url: String,
}

fn default_valkey() -> String {
    "redis://127.0.0.1:6379".to_string()
}

// Implement Default manually to use the default_valkey function
impl Default for HiveConfig {
    fn default() -> Self {
        Self {
            queue: QueueConfig::default(),
            network: NetworkConfig::default(),
            valkey_url: default_valkey(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub channel_capacity: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            batch_size: 500,
            flush_interval_ms: 200,
            channel_capacity: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            timeout_seconds: 30,
        }
    }
}