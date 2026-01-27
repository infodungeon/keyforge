// libs/keyforge-infra/src/net/sync.rs

use crate::error::{InfraError, InfraResult};
use crate::net::client::HiveClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// Represents the global state of assets available on a Hive server.
///
/// The manifest maps relative asset paths (IDs) to their SHA-256 content hashes.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerManifest {
    /// Map of asset ID (e.g., "system/keyboards/corne.json") to its hash.
    pub files: HashMap<String, String>,
}

/// Statistics describing the result of a workspace synchronization operation.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SyncStats {
    /// Number of assets successfully downloaded.
    pub downloaded: usize,
    /// Number of assets merged or updated.
    pub merged: usize,
    /// Number of assets skipped because they were already up-to-date.
    pub skipped: usize,
    /// List of errors encountered during synchronization.
    pub errors: Vec<String>,
}

/// Synchronizes a local workspace with a remote Hive server.
///
/// This fetches the server manifest and downloads any missing or outdated assets.
///
/// # Errors
///
/// Returns an error if the manifest cannot be retrieved.
pub async fn run_sync(client: &HiveClient, root: &Path) -> InfraResult<SyncStats> {
    info!("Starting workspace synchronization...");
    let manifest = client.get_manifest().await?;
    let mut stats = SyncStats::default();

    for (id, remote_hash) in manifest.files {
        let local_path = root.join(&id);

        let needs_download = if local_path.exists() {
            match crate::util::common::calculate_file_hash(&local_path) {
                Ok(local_hash) => local_hash != remote_hash,
                Err(e) => {
                    warn!("Failed to hash local file {}: {}", id, e);
                    true
                }
            }
        } else {
            true
        };

        if needs_download {
            info!("📥 Syncing: {}", id);
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            match client.download_asset_by_path(&id, &local_path).await {
                Ok(()) => stats.downloaded += 1,
                Err(e) => {
                    error!("Failed to download {}: {}", id, e);
                    stats.errors.push(format!("{id}: {e}"));
                }
            }
        } else {
            stats.skipped += 1;
        }
    }

    Ok(stats)
}

/// Bootstraps a new workspace with essential assets required for basic operation.
///
/// # Errors
///
/// Returns an error if the bootstrap manifest cannot be retrieved.
pub async fn bootstrap_essentials(client: &HiveClient, root: &Path) -> InfraResult<Vec<String>> {
    info!("🥾 Bootstrapping essential assets...");
    // For now, we just sync everything in 'system/'
    let stats = run_sync(client, root).await?;
    Ok(stats.errors)
}

/// Generates a manifest of all files under a root directory.
///
/// # Errors
///
/// Returns `InfraError` if file hashing fails.
pub fn generate_manifest(root: &Path) -> InfraResult<ServerManifest> {
    let mut files = HashMap::new();
    let walker = WalkDir::new(root).into_iter().filter_map(Result::ok);

    for entry in walker {
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(root)
                .map_err(|e| InfraError::Io(std::io::Error::other(e)))?
                .to_string_lossy()
                .to_string();

            let hash = crate::util::common::calculate_file_hash(path)?;
            files.insert(relative_path, hash);
        }
    }

    Ok(ServerManifest { files })
}
