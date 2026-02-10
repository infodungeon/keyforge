use crate::error::CommandError;
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_adapter::loader::AssetLoader;
use keyforge_boundary::SafePath;
use keyforge_model::constants::{ASSET_KEYCODES, ASSET_UI_CATEGORIES};
use tauri::AppHandle;

/// Returns the default global application configuration.
#[tauri::command]
#[must_use]
pub fn cmd_get_default_config() -> keyforge_protocol::ConfigDto {
    keyforge_model::config::Config::default().into()
}

/// Retrieves the current keycode registry, either from the active runtime or by loading it from disk.
#[tauri::command]
pub async fn cmd_get_keycodes(
    state: tauri::State<'_, SessionState>,
) -> Result<keyforge_protocol::KeycodeRegistryDto, CommandError> {
    Ok(state
        .assets
        .load::<keyforge_protocol::KeycodeRegistryDto>(ASSET_KEYCODES)
        .await?
        .as_ref()
        .clone())
}

/// Retrieves UI category metadata from local configuration files.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_get_ui_categories(
    app: AppHandle,
    _state: tauri::State<'_, SessionState>,
) -> Result<serde_json::Value, CommandError> {
    let data_dir_buf = get_data_dir(&app)?;

    let data_dir = SafePath::from_trusted_root_path(data_dir_buf);
    let provider = keyforge_infra::FsProvider::new(data_dir);

    let stem = ASSET_UI_CATEGORIES;
    let system_path = provider
        .root()
        .join("system/config")?
        .join(&format!("{stem}.mpk.zst"))?;

    if system_path.as_path().exists() {
        let file = std::fs::File::open(system_path.as_path())?;
        let decoder = zstd::Decoder::new(file)?;
        let json: serde_json::Value = rmp_serde::from_read(decoder)?;
        return Ok(json);
    }

    let user_path = provider.root().join("user/config/ui_categories.json")?;
    if user_path.as_path().exists() {
        let content = keyforge_infra::fs::io::read_to_string_limited(&user_path, 1024 * 1024)?;
        return Ok(serde_json::from_str(&content)?);
    }

    Err(CommandError::Config("ui_categories not found".into()))
}
