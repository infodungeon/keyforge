use keyforge_infra::AssetLoader;
use crate::error::CommandError;
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_model::config::Config;
// use keyforge_protocol::config::Config; // This likely stays Protocol DTO if config passed from FE
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::constants::{ASSET_KEYCODES, ASSET_UI_CATEGORIES};
use tauri::AppHandle;

/// Returns the default global application configuration.
#[tauri::command]
pub fn cmd_get_default_config() -> Config {
    Config::default()
}

/// Retrieves the current keycode registry, either from the active runtime or by loading it from disk.
#[tauri::command]
pub async fn cmd_get_keycodes(
    state: tauri::State<'_, SessionState>,
) -> Result<KeycodeRegistry, CommandError> {
    match state.assets.load::<KeycodeRegistry>(ASSET_KEYCODES).await {
        Ok(reg) => Ok(reg.as_ref().clone()),
        Err(e) => {
            tracing::error!("Failed to load keycodes from disk: {}", e);
            Err(CommandError::Config(format!("Keycodes load failed: {}", e)))
        }
    }
}

/// Retrieves UI category metadata from local configuration files.
#[tauri::command]
pub fn cmd_get_ui_categories(
    app: AppHandle,
    _state: tauri::State<'_, SessionState>,
) -> Result<serde_json::Value, CommandError> {
    let data_dir = get_data_dir(&app).map_err(CommandError::Config)?;
    let provider = keyforge_infra::FsProvider::new(data_dir);

    let stem = ASSET_UI_CATEGORIES;
    let system_path = provider
        .root()
        .join("system/config")
        .join(format!("{}.mpk.zst", stem));

    if system_path.exists() {
        let file = std::fs::File::open(system_path)?;
        let decoder =
            zstd::Decoder::new(file).map_err(|e| CommandError::Internal(e.to_string()))?;
        let json: serde_json::Value =
            rmp_serde::from_read(decoder).map_err(|e| CommandError::Internal(e.to_string()))?;
        return Ok(json);
    }

    let user_path = provider.root().join("user/config/ui_categories.json");
    if user_path.exists() {
        let content = std::fs::read_to_string(user_path)?;
        return Ok(serde_json::from_str(&content)?);
    }

    Err(CommandError::Config("ui_categories not found".into()))
}
