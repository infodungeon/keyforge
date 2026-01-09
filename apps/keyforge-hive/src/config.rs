// apps/keyforge-hive/src/config.rs

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
use std::env;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Database connection string.
    pub database_url: String,
    /// Secret key for internal authentication.
    pub hive_secret: String,
    
    // --- Sub-configs ---
    #[serde(default)]
    pub queue: QueueConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub rate_limits: RateLimitConfig,
    
    /// Connection string for the coordination layer.
    #[serde(default = "default_valkey")]
    pub valkey_url: String,

    /// CORS allowed origins (comma separated or *).
    #[serde(default)]
    pub cors_origins: String,
}

impl AppConfig {
    pub fn load_from_env() -> AppResult<Self> {
        // Critical Requirements
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("Missing required env var: DATABASE_URL".into()))?;
        let hive_secret = env::var("HIVE_SECRET")
            .map_err(|_| AppError::Config("Missing required env var: HIVE_SECRET".into()))?;

        // Optional / Defaulted
        let valkey_url = env::var("VALKEY_URL").unwrap_or_else(|_| default_valkey());
        let cors_origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
        
        // Rate Limits
        let rate_limits = RateLimitConfig::load();

        Ok(Self {
            database_url,
            hive_secret,
            queue: QueueConfig::default(),
            network: NetworkConfig::default(),
            rate_limits,
            valkey_url,
            cors_origins,
        })
    }
}

fn default_valkey() -> String {
    "redis://127.0.0.1:6379".to_string()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub limit_per_sec: u32,
    pub limit_burst: u32,
    pub strict_limit_per_sec: u32,
    pub strict_limit_burst: u32,
}

impl RateLimitConfig {
    fn load() -> Self {
        Self {
            limit_per_sec: parse_env("RATE_LIMIT_PER_SEC", 1000),
            limit_burst: parse_env("RATE_LIMIT_BURST", 2000),
            strict_limit_per_sec: parse_env("STRICT_RATE_LIMIT_PER_SEC", 1),
            strict_limit_burst: parse_env("STRICT_RATE_LIMIT_BURST", 5),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::load()
    }
}

fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}