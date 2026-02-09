// apps/keyforge-hive/src/bootstrap.rs

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

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Minimal bootstrap config that tells Hive where its canonical `data/` root lives.
///
/// This file must live outside of `data/system/config` to avoid a chicken-and-egg
/// dependency. Default location: `/etc/keyforge/hive_bootstrap.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct HiveBootstrapConfig {
    /// Path to the `KeyForge` data root directory.
    ///
    /// Expected layout:
    /// - `${data_root}/system/...` (read-only system assets)
    /// - `${data_root}/user/...`   (server-side writable workspace, if applicable)
    pub data_root: keyforge_model::types::path::SafePath,
}

impl HiveBootstrapConfig {
    /// The canonical filesystem path where the Hive looks for its bootstrap configuration.
    pub const DEFAULT_SYSTEM_PATH: &'static str = "/etc/keyforge/hive.toml";

    /// Returns the resolved path for the bootstrap configuration.
    #[must_use]
    pub fn resolve_path() -> PathBuf {
        // 1. Env Var Override
        if let Ok(p) = std::env::var("KEYFORGE_HIVE_CONFIG") {
            return PathBuf::from(p);
        }

        // 2. User Home (XDG)
        if let Some(mut p) = dirs::config_dir() {
            p.push("keyforge/hive.toml");
            if p.exists() {
                return p;
            }
        }

        // 3. System Fallback
        PathBuf::from(Self::DEFAULT_SYSTEM_PATH)
    }

    /// Loads the bootstrap configuration from the specified TOML file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let safe_path =
            keyforge_model::types::path::SafePath::from_trusted_root_path(path.to_path_buf());
        let raw = keyforge_infra::fs::io::read_to_string_limited(&safe_path, 1024 * 1024)
            .map_err(|e| format!("Failed to read bootstrap config {}: {e}", path.display()))?;
        toml::from_str(&raw)
            .map_err(|e| format!("Failed to parse bootstrap config {}: {e}", path.display()))
    }
}
