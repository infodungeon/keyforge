// apps/keyforge-ui/src-tauri/src/commands/sync.rs
use crate::utils::get_data_dir;
use keyforge_infra::HiveClient;
use keyforge_infra::SyncStats;
use keyforge_infra::{bootstrap_essentials, run_sync};
use tauri::AppHandle;

/// Synchronizes local application data.
#[tauri::command]
pub async fn cmd_sync_data(app: AppHandle, hive_url: String) -> Result<SyncStats, String> {
    let local_data_dir = get_data_dir(&app)?;

    // NOTE: UI currently passes single URL. We default asset URL to port 3001 if hive is 3000, 
    // or just assume they are split. For now, let's derive it or ask the user.
    // Heuristic: If url contains 3000, assume assets on 3001.
    
    let asset_url = hive_url.replace("3000", "3001");

    use keyforge_infra::net::client::ClientConfig;
    let config = ClientConfig {
        api_url: hive_url,
        asset_url,
        secret: None,
        ..Default::default()
    };
    let client = HiveClient::new(config).map_err(|e| e.to_string())?;

    run_sync(&client, &local_data_dir).await
}

#[tauri::command]
pub async fn cmd_bootstrap_assets(app: AppHandle, hive_url: String) -> Result<Vec<String>, String> {
    let local_data_dir = get_data_dir(&app)?;
    let asset_url = hive_url.replace("3000", "3001");

    use keyforge_infra::net::client::ClientConfig;
    let config = ClientConfig {
        api_url: hive_url,
        asset_url,
        secret: None,
        ..Default::default()
    };
    let client = HiveClient::new(config).map_err(|e| e.to_string())?;

    bootstrap_essentials(&client, &local_data_dir).await
}
