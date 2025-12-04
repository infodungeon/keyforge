use crate::models::{ServerManifest, SyncStats};
use crate::utils::get_data_dir;
use keyforge_core::geometry::KeyboardDefinition;
use keyforge_core::util::calculate_file_hash;
use reqwest::Client;
use std::fs;
use std::path::{Component, Path};
use tauri::AppHandle;

#[tauri::command]
pub async fn cmd_sync_data(app: AppHandle, hive_url: String) -> Result<SyncStats, String> {
    let client = Client::new();
    let local_data_dir = get_data_dir(&app)?;

    // Ensure the root data directory exists
    if !local_data_dir.exists() {
        fs::create_dir_all(&local_data_dir).map_err(|e| e.to_string())?;
    }

    // 1. Establish the "Jail" (Canonicalize to resolve any symlinks on the host OS)
    let local_data_root = fs::canonicalize(&local_data_dir)
        .map_err(|e| format!("Failed to canonicalize data root: {}", e))?;

    let manifest_url = format!("{}/manifest", hive_url);
    let server_manifest: ServerManifest = client
        .get(&manifest_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Hive: {}", e))?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut stats = SyncStats {
        downloaded: 0,
        merged: 0,
        skipped: 0,
        errors: vec![],
    };

    for (rel_path, server_hash) in server_manifest.files {
        let path_obj = Path::new(&rel_path);

        // --- SECURITY HARDENING START ---

        // A. Reject absolute paths or paths with '..' components explicitly
        if path_obj.is_absolute()
            || path_obj.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            stats.errors.push(format!(
                "SECURITY WARNING: Server attempted path traversal: {}",
                rel_path
            ));
            continue;
        }

        // B. Construct the target path
        let target_path = local_data_root.join(path_obj);

        // C. Double-check: If parent exists, verify it resolves inside the jail.
        // This prevents attacks where a subdirectory in data/ is actually a symlink to /etc/ or C:\Windows
        if let Some(parent) = target_path.parent() {
            if parent.exists() {
                match fs::canonicalize(parent) {
                    Ok(canon_parent) => {
                        if !canon_parent.starts_with(&local_data_root) {
                            stats.errors.push(format!(
                                "SECURITY WARNING: Symlink escape detected: {}",
                                rel_path
                            ));
                            continue;
                        }
                    }
                    Err(e) => {
                        stats.errors.push(format!(
                            "Filesystem error validating path {}: {}",
                            rel_path, e
                        ));
                        continue;
                    }
                }
            } else {
                // Safe to create directory structure because we validated the components in Step A
                if let Err(e) = fs::create_dir_all(parent) {
                    stats
                        .errors
                        .push(format!("Failed to create directory: {}", e));
                    continue;
                }
            }
        }
        // --- SECURITY HARDENING END ---

        let needs_update = if target_path.exists() {
            let local_hash = calculate_file_hash(&target_path).unwrap_or_default();
            local_hash != server_hash
        } else {
            true
        };

        if needs_update {
            let file_url = format!("{}/data/{}", hive_url, rel_path);
            match client.get(&file_url).send().await {
                Ok(resp) => {
                    if let Ok(content) = resp.bytes().await {
                        // Special handling: Merge Keyboards to preserve local user edits if possible
                        // (Requires reading the file, which we know is safe now due to checks above)
                        if rel_path.ends_with(".json")
                            && rel_path.contains("keyboards")
                            && target_path.exists()
                        {
                            if let Ok(local_content) = fs::read_to_string(&target_path) {
                                if let (Ok(mut local_kb), Ok(server_kb)) = (
                                    serde_json::from_str::<KeyboardDefinition>(&local_content),
                                    serde_json::from_slice::<KeyboardDefinition>(&content),
                                ) {
                                    local_kb.geometry = server_kb.geometry;
                                    local_kb.meta = server_kb.meta;
                                    for (name, layout) in server_kb.layouts {
                                        local_kb.layouts.insert(name, layout);
                                    }
                                    let merged_json =
                                        serde_json::to_string_pretty(&local_kb).unwrap();
                                    if let Err(e) = fs::write(&target_path, merged_json) {
                                        stats.errors.push(format!("Write error: {}", e));
                                    } else {
                                        stats.merged += 1;
                                    }
                                    continue;
                                }
                            }
                        }

                        // Standard overwrite
                        if let Err(e) = fs::write(&target_path, content) {
                            stats.errors.push(format!("Write error: {}", e));
                        } else {
                            stats.downloaded += 1;
                        }
                    }
                }
                Err(e) => stats.errors.push(format!("DL failed: {}", e)),
            }
        } else {
            stats.skipped += 1;
        }
    }
    Ok(stats)
}
