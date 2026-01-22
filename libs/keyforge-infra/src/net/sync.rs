// libs/keyforge-infra/src/net/sync.rs

use crate::net::client::HiveClient;
use crate::net::network::ensure_file;
use crate::util::common::calculate_file_hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
            stats
                .errors
                .push(format!("Normalization failed: {rel_path}"));
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

use keyforge_model::constants::{ASSET_KEYCODES_FILENAME, ASSET_UI_CATEGORIES};

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
    let keycodes_stem = ASSET_KEYCODES_FILENAME
        .strip_suffix(".json")
        .unwrap_or(ASSET_KEYCODES_FILENAME);

    for (rel_path, server_hash) in manifest.files {
        // Robust check for essential system assets
        let is_keyboard_model = rel_path.starts_with("keyboards/models/");
        let is_keycode_def = rel_path.contains(keycodes_stem);
        let is_ui_metadata = rel_path.contains(ASSET_UI_CATEGORIES);

        if is_keyboard_model || is_keycode_def || is_ui_metadata {
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
            let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if filename.starts_with('.') {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::client::ClientConfig;
    use std::fs;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_run_sync() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let content = "asset content";
        let hash = crate::util::common::calculate_file_hash_str(content);

        let manifest = ServerManifest {
            files: [("test.txt".to_string(), hash.clone())]
                .into_iter()
                .collect(),
        };

        Mock::given(method("GET"))
            .and(path("/manifest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifest))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/system/test.txt"))
            .respond_with(ResponseTemplate::new(200).set_body_string(content))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config).unwrap();

        let stats = run_sync(&client, root).await.unwrap();
        assert_eq!(stats.downloaded, 1);
        assert!(root.join("system/test.txt").exists());

        // Skip existing
        let stats = run_sync(&client, root).await.unwrap();
        assert_eq!(stats.skipped, 1);
    }

    #[tokio::test]
    async fn test_run_sync_fail() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let manifest = ServerManifest {
            files: [("fail.txt".to_string(), "wrong_hash".into())]
                .into_iter()
                .collect(),
        };

        Mock::given(method("GET"))
            .and(path("/manifest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifest))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/system/fail.txt"))
            .respond_with(ResponseTemplate::new(200).set_body_string("content"))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config).unwrap();

        let stats = run_sync(&client, root).await.unwrap();
        assert_eq!(stats.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_bootstrap_essentials() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let content = "content";
        let hash = crate::util::common::calculate_file_hash_str(content);

        let manifest = ServerManifest {
            files: [
                ("keyboards/models/test.mpk.zst".to_string(), hash.clone()),
                ("keycodes.mpk.zst".to_string(), hash.clone()),
                ("other.txt".to_string(), hash.clone()),
            ]
            .into_iter()
            .collect(),
        };

        Mock::given(method("GET"))
            .and(path("/manifest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifest))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("content"))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config).unwrap();

        let downloaded = bootstrap_essentials(&client, root).await.unwrap();
        assert_eq!(downloaded.len(), 2);
        assert!(downloaded.contains(&"keycodes.mpk.zst".into()));
    }

    #[test]
    fn test_generate_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("subdir")).unwrap();
        fs::write(root.join("test.txt"), "hello").unwrap();
        fs::write(root.join("subdir/other.txt"), "world").unwrap();
        fs::write(root.join(".hidden"), "secret").unwrap();

        let manifest = generate_manifest(root).unwrap();
        assert_eq!(manifest.files.len(), 2);
        assert!(manifest.files.contains_key("test.txt"));
        assert!(manifest.files.contains_key("subdir/other.txt"));
        assert!(!manifest.files.contains_key(".hidden"));
    }

    #[tokio::test]
    async fn test_run_sync_fail_network() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let manifest = ServerManifest {
            files: [("fail.txt".to_string(), "hash".into())]
                .into_iter()
                .collect(),
        };

        Mock::given(method("GET"))
            .and(path("/manifest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifest))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/system/fail.txt"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config).unwrap();

        let stats = run_sync(&client, root).await.unwrap();
        assert!(!stats.errors.is_empty());
    }
}
