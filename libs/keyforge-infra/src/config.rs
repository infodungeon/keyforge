// Copyright (c) 2025 KeyForge Contributors
//
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
use std::path::{Path, PathBuf};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonConfig {
    pub data_dir: Option<PathBuf>,
    pub hive_url: Option<String>,
    pub logging_level: Option<String>,
    pub cores: Option<usize>,
}

impl CommonConfig {
    /// Loads configuration from a TOML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Loads configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            data_dir: env::var("KEYFORGE_DATA_DIR").ok().map(PathBuf::from),
            hive_url: env::var("KEYFORGE_HIVE_URL").ok(),
            logging_level: env::var("KEYFORGE_LOG").ok(),
            cores: env::var("KEYFORGE_CORES").ok().and_then(|s| s.parse().ok()),
        }
    }

    /// Merges another config into this one, with the other config taking precedence.
    pub fn merge(&mut self, other: Self) {
        if let Some(d) = other.data_dir { self.data_dir = Some(d); }
        if let Some(h) = other.hive_url { self.hive_url = Some(h); }
        if let Some(l) = other.logging_level { self.logging_level = Some(l); }
        if let Some(c) = other.cores { self.cores = Some(c); }
    }

    /// Resolves the final data directory with fallback logic.
    pub fn resolve_data_dir(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| PathBuf::from("."))
    }
}
