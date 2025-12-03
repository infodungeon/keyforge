use crate::models::{ServerManifest, SyncStats};
use crate::utils::get_data_dir;
use keyforge_core::geometry::KeyboardDefinition;
use keyforge_core::util::calculate_file_hash;
use reqwest::Client;
use std::fs;
use tauri::AppHandle;

#[tauri::command]
pub async fn cmd_sync_data(app: AppHandle, hive_url: String) -> Result<SyncStats, String> {
    let client = Client::new();
    let local_data_dir = get_data_dir(&app)?;

    if !local_data_dir.exists() {
        fs::create_dir_all(&local_data_dir).map_err(|e| e.to_string())?;
    }

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
        // SECURITY CHECK
        if rel_path.contains("..") || rel_path.starts_with("/") || rel_path.contains("\\") {
            stats.errors.push(format!("Skipping unsafe path: {}", rel_path));
            continue;
        }

        let local_path = local_data_dir.join(&rel_path);
        if !local_path.starts_with(&local_data_dir) {
            stats.errors.push(format!("Path escaped data root: {}", rel_path));
            continue;
        }

        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        let needs_update = if local_path.exists() {
            let local_hash = calculate_file_hash(&local_path).unwrap_or_default();
            local_hash != server_hash
        } else {
            true
        };

        if needs_update {
            let file_url = format!("{}/data/{}", hive_url, rel_path);
            match client.get(&file_url).send().await {
                Ok(resp) => {
                    if let Ok(content) = resp.bytes().await {
                        // Merge Keyboards
                        if rel_path.ends_with(".json") && rel_path.contains("keyboards") && local_path.exists() {
                            if let Ok(local_content) = fs::read_to_string(&local_path) {
                                if let (Ok(mut local_kb), Ok(server_kb)) = (
                                    serde_json::from_str::<KeyboardDefinition>(&local_content),
                                    serde_json::from_slice::<KeyboardDefinition>(&content),
                                ) {
                                    local_kb.geometry = server_kb.geometry;
                                    local_kb.meta = server_kb.meta;
                                    for (name, layout) in server_kb.layouts {
                                        local_kb.layouts.insert(name, layout);
                                    }
                                    let merged_json = serde_json::to_string_pretty(&local_kb).unwrap();
                                    if let Err(e) = fs::write(&local_path, merged_json) {
                                        stats.errors.push(format!("Write error: {}", e));
                                    } else {
                                        stats.merged += 1;
                                    }
                                    continue;
                                }
                            }
                        }
                        if let Err(e) = fs::write(&local_path, content) {
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