// apps/keyforge-cli/src/update.rs

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


use crate::error::{CliError, Result};
use self_update::cargo_crate_version;

/// Configuration for auto-update feature
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// URL of the update server API endpoint
    pub server_url: String,
    /// Auto-install without confirmation
    pub auto_install: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            server_url: std::env::var("KEYFORGE_UPDATE_URL")
                .unwrap_or_else(|_| "https://keyforge-releases.example.com/api/latest".to_string()),
            auto_install: false,
        }
    }
}

/// Check if a newer version is available
pub async fn check_for_update(config: &UpdateConfig) -> Result<Option<String>> {
    let current_version = cargo_crate_version!();

    // Query server for latest version
    let response = reqwest::get(&config.server_url)
        .await
        .map_err(|e| CliError::Update(format!("Failed to check for updates: {}", e)))?;

    if !response.status().is_success() {
        return Err(CliError::Update(format!(
            "Update server returned error: {}",
            response.status()
        )));
    }

    let update_info: UpdateInfo = response
        .json()
        .await
        .map_err(|e| CliError::Update(format!("Invalid response from update server: {}", e)))?;

    if version_greater_than(&update_info.version, current_version) {
        Ok(Some(update_info.version))
    } else {
        Ok(None)
    }
}

/// Perform binary update
pub fn perform_update(config: &UpdateConfig) -> Result<String> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("your-org") // TODO: Replace with actual org
        .repo_name("keyforge")
        .bin_name("keyforge")
        .current_version(cargo_crate_version!())
        .no_confirm(config.auto_install)
        .build()
        .map_err(|e| CliError::Update(format!("Update configuration failed: {}", e)))?
        .update()
        .map_err(|e| CliError::Update(format!("Update failed: {}", e)))?;

    Ok(status.version().to_string())
}

/// Simple semantic version comparison
fn version_greater_than(v1: &str, v2: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let v1_parts = parse_version(v1);
    let v2_parts = parse_version(v2);

    for (a, b) in v1_parts.iter().zip(v2_parts.iter()) {
        if a > b {
            return true;
        } else if a < b {
            return false;
        }
    }

    v1_parts.len() > v2_parts.len()
}

/// Response from update server
#[derive(Debug, serde::Deserialize)]
struct UpdateInfo {
    version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(version_greater_than("0.9.0", "0.8.0"));
        assert!(version_greater_than("1.0.0", "0.9.9"));
        assert!(!version_greater_than("0.8.0", "0.9.0"));
        assert!(!version_greater_than("0.8.0", "0.8.0"));
        assert!(version_greater_than("0.8.1", "0.8.0"));
    }
}
