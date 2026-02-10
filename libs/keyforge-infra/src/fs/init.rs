// libs/keyforge-infra/src/fs/init.rs

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

use crate::error::{InfraError, InfraResult};
use keyforge_boundary::SafePath;
use keyforge_model::constants::{REQUIRED_ASSETS, SYSTEM_DIRS};
pub use keyforge_model::constants::{USER_RUNTIME_DIRS, USER_WORKSPACE_DIRS};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Defines how the workspace should be handled during startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMode {
    /// Only verify that required system assets exist.
    Validate,
    /// Create the directory structure if missing and verify assets.
    Create,
}

/// Marker file used to identify a valid `KeyForge` workspace.
pub const WORKSPACE_MARKER: &str = ".keyforge_workspace";

/// Orchestrates the setup of the `KeyForge` workspace asynchronously.
///
/// # Errors
/// Returns `InfraError` if directory creation or asset validation fails.
pub async fn initialize_workspace_async(root: &SafePath, mode: InitMode) -> InfraResult<()> {
    info!("Initializing workspace at: {:?}", root);

    if mode == InitMode::Create {
        for dir in SYSTEM_DIRS {
            ensure_dir_async(root, dir).await?;
        }
        for dir in USER_WORKSPACE_DIRS {
            ensure_dir_async(root, dir).await?;
        }
        for dir in USER_RUNTIME_DIRS {
            ensure_dir_async(root, dir).await?;
        }

        let marker = root.as_path().join(WORKSPACE_MARKER);
        if !marker.exists() {
            tokio::fs::write(&marker, "KeyForge Workspace Root\n")
                .await
                .map_err(InfraError::Io)?;
        }
    }

    validate_system_assets(root)?;

    info!("Workspace validation successful.");
    Ok(())
}

/// Orchestrates the setup of the `KeyForge` workspace.
///
/// # Errors
///
/// Returns `InfraError` if directory creation or asset validation fails.
pub fn initialize_workspace(root: &SafePath, mode: InitMode) -> InfraResult<()> {
    info!("Initializing workspace at: {:?}", root);

    if mode == InitMode::Create {
        for dir in SYSTEM_DIRS {
            ensure_dir(root, dir)?;
        }
        for dir in USER_WORKSPACE_DIRS {
            ensure_dir(root, dir)?;
        }
        for dir in USER_RUNTIME_DIRS {
            ensure_dir(root, dir)?;
        }

        // Task-infra-rev-003: Create marker file
        let marker = root.as_path().join(WORKSPACE_MARKER);
        if !marker.exists() {
            fs::write(&marker, "KeyForge Workspace Root\n").map_err(InfraError::Io)?;
        }
    }

    validate_system_assets(root)?;

    info!("Workspace validation successful.");
    Ok(())
}

fn check_asset_exists(system_root: &Path, rel_path: &str) -> bool {
    let bin_path = system_root.join(format!("{rel_path}.mpk.zst"));
    let json_path = system_root.join(format!("{rel_path}.json"));
    bin_path.exists() || json_path.exists()
}

/// Ensures a directory exists relative to the root asynchronously.
///
/// # Errors
/// Returns `InfraError` if directory creation fails.
pub async fn ensure_dir_async(root: &SafePath, rel_path: &str) -> InfraResult<PathBuf> {
    let p = root.as_path().join(rel_path);
    if !p.exists() {
        tokio::fs::create_dir_all(&p)
            .await
            .map_err(InfraError::Io)?;
        info!("   Created: {:?}", p);
    }
    Ok(p)
}

/// Ensures a directory exists relative to the root, creating it if necessary.
///
/// # Errors
///
/// Returns `InfraError` if directory creation fails.
pub fn ensure_dir(root: &SafePath, rel_path: &str) -> InfraResult<PathBuf> {
    let p = root.as_path().join(rel_path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(InfraError::Io)?;
        info!("   Created: {:?}", p);
    }
    Ok(p)
}

/// Validates that all critical system assets are present in the workspace.
///
/// # Errors
/// Returns `InfraError::Config` if any required asset is missing.
pub fn validate_system_assets(root: &SafePath) -> InfraResult<()> {
    let system_root = root.as_path().join("system");
    for asset in REQUIRED_ASSETS {
        if !check_asset_exists(&system_root, asset) {
            let msg = format!("FATAL: Required system asset missing: {asset}");
            error!("{}", msg);
            return Err(InfraError::Config(msg));
        }
    }
    Ok(())
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_init_workspace_creates_dirs() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("new_workspace");

        let sys_root = root.join("system");
        fs::create_dir_all(sys_root.join("config"))?;
        fs::create_dir_all(sys_root.join("weights"))?;
        fs::create_dir_all(sys_root.join("corpora/text/en_std"))?;

        fs::write(sys_root.join("config/keycodes.json"), "")?;
        fs::write(sys_root.join("weights/cost_matrix.json"), "")?;
        fs::write(sys_root.join("corpora/text/en_std/1grams.json"), "")?;

        let dot = SafePath::try_from_str(".")?;
        let safe_root = SafePath::from_trusted_root(&root, &dot);
        initialize_workspace(&safe_root, InitMode::Create)?;

        assert!(root.join("system/config").exists());
        assert!(root.join("user/keyboards").exists());
        assert!(root.join("user/agent_wal").exists());
        Ok(())
    }

    #[test]
    fn test_init_workspace_missing_assets() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();

        let dot = SafePath::try_from_str(".")?;
        let safe_root = SafePath::from_trusted_root(root, &dot);
        let res = initialize_workspace(&safe_root, InitMode::Validate);
        assert!(res.is_err());
        assert!(format!("{:?}", res.err()).contains("Required system asset missing"));
        Ok(())
    }
}
