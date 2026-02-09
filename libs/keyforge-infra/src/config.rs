// libs/keyforge-infra/src/config.rs

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

use keyforge_model::constants::DEFAULT_FALLBACK_PATH;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// Environment variable key for the data directory path.
pub const ENV_DATA_DIR: &str = "KEYFORGE_DATA_DIR";
/// Environment variable key for the Hive server URL.
pub const ENV_HIVE_URL: &str = "KEYFORGE_HIVE_URL";
/// Environment variable key for the logging level.
pub const ENV_LOG_LEVEL: &str = "KEYFORGE_LOG";
/// Environment variable key for the number of CPU cores to use.
pub const ENV_CORES: &str = "KEYFORGE_CORES";

/// Common infrastructure configuration shared across `KeyForge` applications.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonConfig {
    /// Root directory for local application data.
    pub data_dir: Option<PathBuf>,
    /// Base URL for the remote Hive server.
    pub hive_url: Option<String>,
    /// Desired logging level (e.g., "info", "debug").
    pub logging_level: Option<String>,
    /// Number of CPU cores to utilize for parallel processing.
    pub cores: Option<usize>,
}

impl CommonConfig {
    /// Loads configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an `InfraError` if the file cannot be read or parsed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::error::InfraResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Loads configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            data_dir: env::var(ENV_DATA_DIR).ok().map(PathBuf::from),
            hive_url: env::var(ENV_HIVE_URL).ok(),
            logging_level: env::var(ENV_LOG_LEVEL).ok(),
            cores: env::var(ENV_CORES).ok().and_then(|s| s.parse().ok()),
        }
    }

    /// Merges another config into this one, with the other config taking precedence.
    pub fn merge(&mut self, other: Self) {
        if let Some(d) = other.data_dir {
            self.data_dir = Some(d);
        }
        if let Some(h) = other.hive_url {
            self.hive_url = Some(h);
        }
        if let Some(l) = other.logging_level {
            self.logging_level = Some(l);
        }
        if let Some(c) = other.cores {
            self.cores = Some(c);
        }
    }

    /// Resolves the final data directory with fallback logic.
    #[must_use]
    pub fn resolve_data_dir(&self) -> PathBuf {
        if let Some(d) = &self.data_dir {
            return d.clone();
        }

        if let Ok(d) = env::var(ENV_DATA_DIR) {
            return PathBuf::from(d);
        }

        // Prioritize local "data" directory if present (common in dev/repo root)
        let local_data = PathBuf::from("data");
        if local_data.exists() && local_data.is_dir() {
            return local_data;
        }

        // Use standard OS data directory as fallback
        if let Some(mut d) = dirs::data_dir() {
            d.push("keyforge");
            return d;
        }

        PathBuf::from(DEFAULT_FALLBACK_PATH)
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_common_config_from_env() {
        temp_env::with_vars(
            vec![(ENV_DATA_DIR, Some("/tmp/data")), (ENV_CORES, Some("8"))],
            || {
                let cfg = CommonConfig::from_env();
                assert_eq!(cfg.data_dir, Some(PathBuf::from("/tmp/data")));
                assert_eq!(cfg.cores, Some(8));
            },
        );
    }

    #[test]
    fn test_common_config_from_file() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("config.toml");
        fs::write(&path, "data_dir = '/etc/kf'\ncores = 4")?;

        let cfg = CommonConfig::from_file(&path)?;
        assert_eq!(cfg.data_dir, Some(PathBuf::from("/etc/kf")));
        assert_eq!(cfg.cores, Some(4));

        assert!(CommonConfig::from_file("missing.toml").is_err());
        fs::write(&path, "bad = invalid")?;
        assert!(CommonConfig::from_file(&path).is_err());
        Ok(())
    }

    #[test]
    fn test_common_config_merge() {
        let mut cfg1 = CommonConfig {
            data_dir: Some(PathBuf::from("/a")),
            cores: Some(1),
            ..Default::default()
        };
        let cfg2 = CommonConfig {
            data_dir: Some(PathBuf::from("/b")),
            logging_level: Some("debug".into()),
            ..Default::default()
        };

        cfg1.merge(cfg2);
        assert_eq!(cfg1.data_dir, Some(PathBuf::from("/b")));
        assert_eq!(cfg1.cores, Some(1));
        assert_eq!(cfg1.logging_level, Some("debug".into()));
    }

    #[test]
    fn test_common_config_resolve() {
        temp_env::with_var(ENV_DATA_DIR, Some("/tmp/keyforge_test"), || {
            let mut cfg = CommonConfig::default();
            assert_eq!(cfg.resolve_data_dir(), PathBuf::from("/tmp/keyforge_test"));

            cfg.data_dir = Some(PathBuf::from("/custom"));
            assert_eq!(cfg.resolve_data_dir(), PathBuf::from("/custom"));
        });
    }
}
