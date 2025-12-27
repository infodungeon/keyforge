// libs/keyforge-infra/src/fs/init.rs
use crate::error::{InfraError, InfraResult};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMode {
    /// Strict Mode: Validates system assets exist. Auto-creates full user environment (Workspace + Runtime).
    Validate,
    /// Setup Mode: Creates skeleton structure for CLI (System + Workspace). Skips Runtime dirs.
    Provision,
}

const REQUIRED_ASSETS: &[&str] = &[
    "config/keycodes",
    "weights/cost_matrix",
    "corpora/text/en_std/1grams",
];

const SYSTEM_DIRS: &[&str] = &[
    "system/config",
    "system/keyboards",
    "system/corpora/text/en_std",
    "system/weights",
    "system/benchmarks",
];

// Directories needed for Logic/Analysis (CLI + Server + Agent)
const USER_WORKSPACE_DIRS: &[&str] = &[
    "user/keyboards",
    "user/corpora",
    "user/weights",
    "user/config",
];

// Directories needed only for Execution/Persistence (Server + Agent)
const USER_RUNTIME_DIRS: &[&str] = &["user/queue", "user/agent_wal", "user/temp"];

fn check_asset_exists(system_root: &Path, rel_path: &str) -> bool {
    // Check for binary (.mpk.zst) OR json (.json)
    let bin_path = system_root.join(format!("{}.mpk.zst", rel_path));
    let json_path = system_root.join(format!("{}.json", rel_path));
    bin_path.exists() || json_path.exists()
}

fn ensure_dir(root: &Path, rel_path: &str, created: &mut Vec<PathBuf>) -> InfraResult<()> {
    let p = root.join(rel_path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(InfraError::Io)?;
        info!("   Created: {:?}", p);
        created.push(p);
    }
    Ok(())
}

pub fn initialize_workspace(root: &Path, mode: InitMode) -> InfraResult<Vec<PathBuf>> {
    let mut created = Vec::new();
    let system_root = root.join("system");

    match mode {
        InitMode::Validate => {
            // 1. Guard Dog: Check System Assets
            for asset in REQUIRED_ASSETS {
                if !check_asset_exists(&system_root, asset) {
                    let msg = format!("FATAL: Required system asset missing: {}", asset);
                    error!("{}", msg); // System Log
                    eprintln!("{}", msg); // Stderr
                    return Err(InfraError::Config(msg)); // Exception
                }
            }

            // 2. Ensure User Environment (Workspace + Runtime)
            // Server/Agent need queues and WALs to function.
            for d in USER_WORKSPACE_DIRS.iter().chain(USER_RUNTIME_DIRS.iter()) {
                ensure_dir(root, d, &mut created)?;
            }
        }
        InitMode::Provision => {
            // CLI Init: Create System skeletons + User Workspace.
            // No Runtime dirs needed for offline CLI analysis.
            for d in SYSTEM_DIRS.iter().chain(USER_WORKSPACE_DIRS.iter()) {
                ensure_dir(root, d, &mut created)?;
            }
        }
    }

    Ok(created)
}
