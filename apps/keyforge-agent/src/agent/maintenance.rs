// apps/keyforge-agent/src/agent/maintenance.rs

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

use crate::models::MaintenanceConfig;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::{info, warn};

const SAFE_EXTENSION: &str = "json";

/// Prunes stale user data.
pub async fn prune_stale_data(
    data_root: PathBuf,
    config: &MaintenanceConfig,
) -> std::io::Result<()> {
    // Target directory from config
    let target_dir = data_root.join(&config.prune_target_dir);
    if !target_dir.exists() {
        return Ok(());
    }

    let ttl_secs = config.ttl_days * 86400;
    let cutoff = SystemTime::now() - Duration::from_secs(ttl_secs);
    let mut entries = fs::read_dir(&target_dir).await?;
    let mut count = 0;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        // 1. Extension Check (Safety)
        if path.extension().and_then(|s| s.to_str()) != Some(SAFE_EXTENSION) {
            continue;
        }

        // 2. Age Check
        if let Ok(meta) = entry.metadata().await {
            if let Ok(modified) = meta.modified() {
                if modified < cutoff {
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    info!(file = %file_name, "pruning stale user asset");
                    if let Err(e) = fs::remove_file(&path).await {
                        warn!(file = %file_name, error = %e, "failed to delete stale asset");
                    } else {
                        count += 1;
                    }
                }
            }
        }
    }

    if count > 0 {
        info!(count = count, "maintenance complete");
    }
    Ok(())
}
