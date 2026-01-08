// ===== keyforge/ui/src-tauri/src/commands/sync.rs =====
use crate::utils::get_data_dir;
use keyforge_infra::HiveClient;
use keyforge_infra::SyncStats;
use keyforge_infra::{bootstrap_essentials, run_sync};
use tauri::AppHandle;

/// Synchronizes local application data with a remote Hive server.
#[tauri::command]
pub async fn cmd_sync_data(app: AppHandle, hive_url: String) -> Result<SyncStats, String> {
    let local_data_dir = get_data_dir(&app)?;

    // No secret needed for public sync
    let client = HiveClient::new(hive_url, None).map_err(|e| e.to_string())?;

    run_sync(&client, &local_data_dir).await
}

/// Bootstraps essential application assets from a remote Hive server.
#[tauri::command]
pub async fn cmd_bootstrap_assets(app: AppHandle, hive_url: String) -> Result<Vec<String>, String> {
    let local_data_dir = get_data_dir(&app)?;

    let client = HiveClient::new(hive_url, None).map_err(|e| e.to_string())?;

    bootstrap_essentials(&client, &local_data_dir).await
}
