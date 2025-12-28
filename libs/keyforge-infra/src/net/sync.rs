use crate::net::client::HiveClient;
use crate::net::network::ensure_file;
use crate::util::common::calculate_file_hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path};
use tracing::{error, info};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub async fn run_sync(client: &HiveClient, local_data_root: &Path) -> Result<SyncStats, String> {
    info!("🔄 Starting Sync...");
    let server_manifest: ServerManifest = client
        .get("manifest")
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Hive: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;

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
    let jail =
        fs::canonicalize(&system_root).map_err(|e| format!("Invalid local system root: {}", e))?;

    for (rel_path, server_hash) in server_manifest.files {
        let path_obj = Path::new(&rel_path);
        if path_obj.is_absolute()
            || path_obj.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
            || rel_path.contains('\\')
        {
            let msg = format!(
                "SECURITY WARNING: Server attempted path traversal: {}",
                rel_path
            );
            error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        let target_path = jail.join(path_obj);
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }

        let needs_update = if target_path.exists() {
            calculate_file_hash(&target_path).unwrap_or_default() != server_hash
        } else {
            true
        };

        if needs_update {
            let remote_url = client.url(&format!("data/system/{}", rel_path));
            match ensure_file(client, &remote_url, &target_path, Some(&server_hash)).await {
                Ok(_) => stats.downloaded += 1,
                Err(e) => stats
                    .errors
                    .push(format!("Sync failed for {}: {}", rel_path, e)),
            }
        } else {
            stats.skipped += 1;
        }
    }
    Ok(stats)
}

pub async fn bootstrap_essentials(
    client: &HiveClient,
    local_root: &Path,
) -> Result<Vec<String>, String> {
    info!("🚀 Bootstrapping essential assets (Binary Format)...");
    let mut downloaded = Vec::new();

    let keyboards = ["ortho_30", "ansi_104", "corne", "szr35"];
    for kb in keyboards {
        let filename = format!("{}.mpk.zst", kb);
        // Updated to reflect new directory structure
        let remote = client.url(&format!("data/system/keyboards/models/{}", filename));
        let local = local_root.join("system/keyboards/models").join(&filename);
        if ensure_file(client, &remote, &local, None).await.is_ok() {
            downloaded.push(kb.to_string());
        }
    }

    let configs = ["keycodes.mpk.zst", "ui_categories.mpk.zst"];
    for cfg in configs {
        let remote = client.url(&format!("data/system/config/{}", cfg));
        let local = local_root.join("system/config").join(cfg);
        if ensure_file(client, &remote, &local, None).await.is_ok() {
            downloaded.push(cfg.to_string());
        }
    }
    Ok(downloaded)
}

pub fn generate_manifest(data_root: &Path) -> crate::error::InfraResult<ServerManifest> {
    let mut files = HashMap::new();

    // Scan entire data_root recursively, following symlinks
    let walker = WalkDir::new(data_root).follow_links(true);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();

            // Skip hidden files/directories
            if path
                .components()
                .any(|c| matches!(c, Component::Normal(s) if s.to_string_lossy().starts_with('.')))
            {
                continue;
            }

            // Skip benchmarks directory
            if path
                .components()
                .any(|c| matches!(c, Component::Normal(s) if s.to_string_lossy() == "benchmarks"))
            {
                continue;
            }

            // Skip testing artifacts
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("testing."))
                .unwrap_or(false)
            {
                continue;
            }

            if let Ok(hash) = calculate_file_hash(path) {
                if let Ok(relative) = path.strip_prefix(data_root) {
                    // println!("✅ Added to Manifest: {}", relative.to_string_lossy());
                    files.insert(relative.to_string_lossy().replace('\\', "/"), hash);
                }
            }
        }
    }
    Ok(ServerManifest { files })
}
