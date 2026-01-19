// libs/keyforge-infra/src/net/sync.rs

use crate::net::client::HiveClient;
use crate::net::network::ensure_file;
use crate::util::common::calculate_file_hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path};
use tracing::info;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ServerManifest {
    pub files: HashMap<String, String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SyncStats {
    pub downloaded: usize,
    pub merged: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// Orchestrates the synchronization of system assets between Hive and Local Data.
///
/// # Errors
///
/// Returns an error string if metadata fetching or file downloading fails.
pub async fn run_sync(client: &HiveClient, local_data_root: &Path) -> Result<SyncStats, String> {
    info!("🔄 Starting Sync...");
    // Manifest is served from Asset Server
    let url = client.asset_url("manifest");

    let op = || async {
        client
            .inner()
            .get(&url)
            .send()
            .await
            .map_err(|e| backoff::Error::transient(format!("Failed to fetch manifest: {e}")))?
            .json::<ServerManifest>()
            .await
            .map_err(|e| backoff::Error::permanent(format!("Invalid manifest JSON: {e}")))
    };

    let backoff_conf = backoff::ExponentialBackoff {
        max_elapsed_time: Some(std::time::Duration::from_secs(60)),
        ..Default::default()
    };

    let server_manifest = backoff::future::retry(backoff_conf, op).await?;

    let mut stats = SyncStats {
        downloaded: 0,
        merged: 0,
        skipped: 0,
        errors: vec![],
    };
    let system_root = local_data_root.join("system");
    if !system_root.exists() {
        fs::create_dir_all(&system_root).map_err(|e| e.to_string())?;
    }
    let jail = fs::canonicalize(&system_root).map_err(|e| e.to_string())?;

    for (rel_path, server_hash) in server_manifest.files {
        let Some(normalized) = crate::util::common::normalize_path(&rel_path) else {
            continue;
        };

        let target_path = jail.join(normalized);
        let needs_update = if target_path.exists() {
            calculate_file_hash(&target_path).unwrap_or_default() != server_hash
        } else {
            true
        };

        if needs_update {
            let remote_url = client.asset_url(&format!("data/system/{rel_path}"));
            match ensure_file(client, &remote_url, &target_path, Some(&server_hash)).await {
                Ok(()) => stats.downloaded += 1,
                Err(e) => stats.errors.push(format!("{rel_path}: {e}")),
            }
        } else {
            stats.skipped += 1;
        }
    }
    Ok(stats)
}

/// Bootstraps essential assets (Config/Keycodes) required for basic operation.
///
/// # Errors
///
/// Returns an error string if essential assets fail to download.
pub async fn bootstrap_essentials(
    client: &HiveClient,
    local_root: &Path,
) -> Result<Vec<String>, String> {
    info!("🚀 Bootstrapping essential assets...");
    let url = client.asset_url("manifest");
    let manifest: ServerManifest = client
        .inner()
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut downloaded = Vec::new();
    for (rel_path, server_hash) in manifest.files {
        let is_keyboard =
            rel_path.starts_with("keyboards/models/") && rel_path.ends_with(".mpk.zst");
        let is_keycodes = rel_path.contains("keycodes.mpk.zst");
        let is_cats = rel_path.contains("ui_categories.mpk.zst");

        if is_keyboard || is_keycodes || is_cats {
            let remote = client.asset_url(&format!("data/system/{rel_path}"));
            let local = local_root.join("system").join(&rel_path);
            if ensure_file(client, &remote, &local, Some(&server_hash))
                .await
                .is_ok()
            {
                downloaded.push(rel_path);
            }
        }
    }
    Ok(downloaded)
}

/// Generates a manifest of local assets for comparison with Hive.
///
/// # Errors
///
/// Returns `InfraError` if file scanning or hashing fails.
pub fn generate_manifest(data_root: &Path) -> crate::error::InfraResult<ServerManifest> {
    let mut files = HashMap::new();
    let walker = WalkDir::new(data_root).follow_links(true);

    for entry in walker.into_iter().filter_map(std::result::Result::ok) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path
                .components()
                .any(|c| matches!(c, Component::Normal(s) if s.to_string_lossy().starts_with('.')))
            {
                continue;
            }

            if let Ok(hash) = calculate_file_hash(path) {
                if let Ok(relative) = path.strip_prefix(data_root) {
                    files.insert(relative.to_string_lossy().replace('\\', "/"), hash);
                }
            }
        }
    }
    Ok(ServerManifest { files })
}
