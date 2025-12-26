use crate::error::{InfraError, InfraResult};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub fn initialize_workspace(root: &Path) -> InfraResult<Vec<PathBuf>> {
    let mut created = Vec::new();

    // 1. Define System Directories (Read-Only / Standard Library)
    let system_dirs = [
        "system/config",
        "system/keyboards",
        "system/corpora/text/en_std",
        "system/weights",
        "system/benchmarks",
    ];

    // 2. Define User Directories (Read-Write / Workspace)
    let user_dirs = [
        "user/keyboards",
        "user/corpora",
        "user/weights",
        "user/config",
        "user/queue",
        "user/agent_wal",
        "user/temp",
    ];

    for d in system_dirs.iter().chain(user_dirs.iter()) {
        let p = root.join(d);
        if !p.exists() {
            fs::create_dir_all(&p).map_err(InfraError::Io)?;
            info!("   Created: {:?}", p);
            created.push(p);
        }
    }

    Ok(created)
}
