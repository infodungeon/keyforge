use crate::utils::get_data_dir;
use keyforge_core::api::KeyForgeState;
use keyforge_core::config::Config;
use keyforge_core::keycodes::KeycodeRegistry;
use tauri::AppHandle;

#[tauri::command]
pub fn cmd_get_default_config() -> Config {
    Config::default()
}

#[tauri::command]
pub fn cmd_get_keycodes(state: tauri::State<KeyForgeState>) -> Result<KeycodeRegistry, String> {
    // FIXED: .lock() -> .read()
    let sessions = state.sessions.read().map_err(|e| e.to_string())?;

    if let Some(session) = sessions.get("primary") {
        Ok(session.registry.clone())
    } else {
        Ok(KeycodeRegistry::new_with_defaults())
    }
}

#[tauri::command]
pub fn cmd_get_ui_categories(app: AppHandle) -> Result<serde_json::Value, String> {
    let data_dir = get_data_dir(&app)?;
    let path = data_dir.join("ui_categories.json");

    if !path.exists() {
        return Err("ui_categories.json not found in data directory".into());
    }

    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(json)
}
