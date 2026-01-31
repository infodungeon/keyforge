// apps/keyforge-hive/src/config.rs

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

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::env;
use zeroize::Zeroize;

// --- Defaults ---
pub const DEFAULT_POPULATION_LIMIT: usize = 50;
pub const DEFAULT_VALKEY_URL: &str = "redis://127.0.0.1:6379";
pub const DEFAULT_QUEUE_BATCH_SIZE: usize = 500;
pub const DEFAULT_QUEUE_FLUSH_INTERVAL_MS: u64 = 200;
pub const DEFAULT_QUEUE_CHANNEL_CAPACITY: usize = 1000;
pub const DEFAULT_NETWORK_MAX_CONNECTIONS: u32 = 100;
pub const DEFAULT_NETWORK_TIMEOUT_SECONDS: u64 = 30;
pub const DEFAULT_RATE_LIMIT_PER_SEC: u32 = 1000;
pub const DEFAULT_RATE_LIMIT_BURST: u32 = 2000;
pub const DEFAULT_STRICT_RATE_LIMIT_PER_SEC: u32 = 5;
pub const DEFAULT_STRICT_RATE_LIMIT_BURST: u32 = 10;
pub const DEFAULT_API_KEY_CACHE_CAPACITY: u64 = 1000;
pub const DEFAULT_API_KEY_CACHE_TTL_SECS: u64 = 300;
pub const DEFAULT_NONCE_CACHE_CAPACITY: u64 = 100_000;
pub const DEFAULT_NONCE_CACHE_TTL_SECS: u64 = 600;
pub const DEFAULT_SUBMISSION_EXPIRATION_SECS: u64 = 3600;
pub const DEFAULT_BROADCAST_CAPACITY: usize = 10000;
pub const DEFAULT_MONITOR_INTERVAL_SECS: u64 = 5;
pub const DEFAULT_MAX_CONCURRENT_COMPILATIONS: usize = 4;

/// The global application configuration, aggregating settings for database, network, queue, and rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct AppConfig {
    /// Database connection string.
    pub database_url: String,
    /// Secret key for internal authentication.
    pub hive_secret: String,

    // --- Sub-configs ---
    /// Configuration for the background job processing queue.
    #[serde(default)]
    pub queue: QueueConfig,
    /// Configuration for network timeouts and connection limits.
    #[serde(default)]
    pub network: NetworkConfig,
    /// Configuration for API rate limiting policies.
    #[serde(default)]
    pub rate_limits: RateLimitConfig,

    /// Connection string for the coordination layer.
    #[serde(default = "default_valkey")]
    pub valkey_url: String,

    /// CORS allowed origins (comma separated or *).
    #[serde(default)]
    pub cors: keyforge_model::config::CorsConfig,

    /// Unique Server Identity Key (Machine ID).
    /// If not provided, one will be generated Ephemerally (WARNING: unstable across restarts).
    #[serde(default)]
    pub server_key: Option<String>,

    /// Maximum number of layouts to keep per job in the population.
    #[serde(default = "default_population_limit")]
    pub population_limit: usize,

    /// Maximum number of concurrent engine compilations for verification.
    #[serde(default = "default_max_concurrent_compilations")]
    pub max_concurrent_compilations: usize,
}

fn default_population_limit() -> usize {
    DEFAULT_POPULATION_LIMIT
}

fn default_max_concurrent_compilations() -> usize {
    DEFAULT_MAX_CONCURRENT_COMPILATIONS
}

fn default_valkey() -> String {
    DEFAULT_VALKEY_URL.to_string()
}

impl AppConfig {
    /// Loads the configuration from environment variables.
    /// Returns an error if critical variables (`DATABASE_URL`, `HIVE_SECRET`) are missing.
    pub fn load_from_env() -> AppResult<Self> {
        // Critical Requirements - Fail Fast
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("Missing required env var: DATABASE_URL".into()))?;
        let hive_secret = env::var("HIVE_SECRET")
            .map_err(|_| AppError::Config("Missing required env var: HIVE_SECRET".into()))?;

        // Optional / Defaulted
        let valkey_url = env::var("KEYFORGE_VALKEY_URL")
            .or_else(|_| env::var("VALKEY_URL"))
            .unwrap_or_else(|_| default_valkey());

        let cors = keyforge_model::config::CorsConfig {
            allowed_origins: env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default(),
        };
        let server_key = env::var("HIVE_SERVER_KEY").ok();
        let population_limit = parse_env("POPULATION_LIMIT", DEFAULT_POPULATION_LIMIT);
        let max_concurrent_compilations = parse_env(
            "MAX_CONCURRENT_COMPILATIONS",
            DEFAULT_MAX_CONCURRENT_COMPILATIONS,
        );

        // Rate Limits
        let rate_limits = RateLimitConfig::load();

        Ok(Self {
            database_url,
            hive_secret,
            queue: QueueConfig::load(),
            network: NetworkConfig::load(),
            rate_limits,
            valkey_url,

            cors,
            server_key,
            population_limit,
            max_concurrent_compilations,
        })
    }

    /// Creates a default configuration for testing.
    #[must_use]
    pub fn mock() -> Self {
        Self {
            database_url: "postgres://mock".to_string(),
            hive_secret: "mock_secret".to_string(),
            queue: QueueConfig::default(),
            network: NetworkConfig::default(),
            rate_limits: RateLimitConfig::default(),
            valkey_url: DEFAULT_VALKEY_URL.to_string(),
            cors: keyforge_model::config::CorsConfig {
                allowed_origins: "*".to_string(),
            },
            server_key: Some("mock_server_key".to_string()),
            population_limit: DEFAULT_POPULATION_LIMIT,
            max_concurrent_compilations: DEFAULT_MAX_CONCURRENT_COMPILATIONS,
        }
    }
}

/// Configuration settings for the job queue system.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct QueueConfig {
    /// The number of items to process in a single batch transaction.
    pub batch_size: usize,
    /// The interval in milliseconds between forced queue flushes.
    pub flush_interval_ms: u64,
    /// The maximum number of pending items allowed in the in-memory channel.
    pub channel_capacity: usize,
}

impl QueueConfig {
    fn load() -> Self {
        Self {
            batch_size: parse_env("QUEUE_BATCH_SIZE", DEFAULT_QUEUE_BATCH_SIZE),
            flush_interval_ms: parse_env("QUEUE_FLUSH_INTERVAL", DEFAULT_QUEUE_FLUSH_INTERVAL_MS),
            channel_capacity: parse_env("QUEUE_CAPACITY", DEFAULT_QUEUE_CHANNEL_CAPACITY),
        }
    }
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self::load()
    }
}

/// Network usage parameters for controlling resource consumption.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct NetworkConfig {
    /// The maximum number of concurrent database or network connections allowed.
    pub max_connections: u32,
    /// The default timeout in seconds for network operations.
    pub timeout_seconds: u64,
}

impl NetworkConfig {
    fn load() -> Self {
        Self {
            max_connections: parse_env("MAX_CONNECTIONS", DEFAULT_NETWORK_MAX_CONNECTIONS),
            timeout_seconds: parse_env("NETWORK_TIMEOUT", DEFAULT_NETWORK_TIMEOUT_SECONDS),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::load()
    }
}

/// Rate limiting parameters for API protection.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct RateLimitConfig {
    /// The standard number of requests allowed per second per IP.
    pub limit_per_sec: u32,
    /// The burst allowance for standard requests.
    pub limit_burst: u32,
    /// The stricter request limit per second for sensitive endpoints (e.g., job registration).
    pub strict_limit_per_sec: u32,
    /// The burst allowance for sensitive endpoints.
    pub strict_limit_burst: u32,
}

impl RateLimitConfig {
    fn load() -> Self {
        Self {
            limit_per_sec: parse_env("RATE_LIMIT_PER_SEC", DEFAULT_RATE_LIMIT_PER_SEC),
            limit_burst: parse_env("RATE_LIMIT_BURST", DEFAULT_RATE_LIMIT_BURST),
            strict_limit_per_sec: parse_env(
                "STRICT_RATE_LIMIT_PER_SEC",
                DEFAULT_STRICT_RATE_LIMIT_PER_SEC,
            ),
            strict_limit_burst: parse_env(
                "STRICT_RATE_LIMIT_BURST",
                DEFAULT_STRICT_RATE_LIMIT_BURST,
            ),
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
