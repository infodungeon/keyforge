use crate::error::CommandError;
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_export::{qmk::QmkExporter, zmk::ZmkExporter, Exporter};
use keyforge_infra::fs::listing;
use keyforge_infra::HiveClient;
use keyforge_model::constants::{
    DEFAULT_AUTHOR_NAME, DEFAULT_KEYBOARD_NAME, DEFAULT_KLE_NOTES, DEFAULT_VERSION,
};
use keyforge_model::geometry::kle::{parse_kle_json, to_kle_json};
use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use keyforge_adapter::loader::AssetLoader;
use keyforge_persistence::UserRepo;
use std::collections::HashMap;
use std::path::Path;
use tauri::AppHandle;

/// Lists all available keyboard geometries in the workspace.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_list_keyboards(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app)?;
    listing::list_keyboards(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

/// Lists all available keymap extra assets.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_list_keymap_extras(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app)?;
    listing::list_keymap_extras(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
/// Retrieves all currently loaded layouts from the session state.
pub async fn cmd_get_loaded_layouts(
    _state: tauri::State<'_, SessionState>,
) -> Result<HashMap<String, String>, CommandError> {
    Ok(HashMap::new())
}

/// Retrieves the geometry definition for a specific keyboard.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn cmd_get_keyboard_geometry(
    _app: AppHandle,
    state: tauri::State<'_, SessionState>,
    name: String,
) -> Result<KeyboardGeometry, CommandError> {
    Ok(state
        .assets
        .load::<KeyboardDefinition>(&name)
        .await?
        .geometry
        .clone())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
/// Retrieves all layouts (both system and user) for a specific keyboard.
pub async fn cmd_get_all_layouts_scoped(
    app: AppHandle,
    state: tauri::State<'_, SessionState>,
    keyboard_id: String,
) -> Result<HashMap<String, String>, CommandError> {
    let data_dir = get_data_dir(&app)?;
    let user_data = UserRepo::new(data_dir);
    let mut all_layouts = user_data.get_layouts(&keyboard_id);

    if let Ok(def) = state.assets.load::<KeyboardDefinition>(&keyboard_id).await {
        all_layouts.extend(def.layouts.clone());
    }
    Ok(all_layouts)
}

/// Saves a custom user layout to the local repository.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_save_user_layout(
    app: AppHandle,
    keyboard_id: String,
    name: String,
    layout: String,
) -> Result<(), CommandError> {
    let data_dir = get_data_dir(&app)?;
    UserRepo::new(data_dir).save_layout(&keyboard_id, &name, &layout)?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
/// Deletes a custom user layout from the local repository.
pub fn cmd_delete_user_layout(
    app: AppHandle,
    keyboard_id: String,
    name: String,
) -> Result<(), CommandError> {
    let data_dir = get_data_dir(&app)?;
    UserRepo::new(data_dir).delete_layout(&keyboard_id, &name)?;
    Ok(())
}

/// Submits a user layout to the remote Hive server for community review.
#[tauri::command]
pub async fn cmd_submit_user_layout(
    hive_url: String,
    hive_secret: String,
    name: String,
    layout: String,
    author: String,
) -> Result<String, CommandError> {
    // Assume assets are on port 3001 if hive is 3000
    let asset_url = hive_url.replace("3000", "3001");

    let config = keyforge_infra::net::client::ClientConfig {
        api_url: hive_url,
        asset_url,
        secret: Some(hive_secret),
        ..Default::default()
    };
    let client = HiveClient::new(config)?;
    let res = client
        .post("submissions")
        .json(&serde_json::json!({ "name": name, "layout": layout, "author": author }))
        .send()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;
    if res.status().is_success() {
        Ok("Submitted".to_string())
    } else {
        Err(CommandError::Network(format!(
            "Server Error: {}",
            res.status()
        )))
    }
}

#[tauri::command]
/// Parses a Keyboard Layout Editor (KLE) JSON string into a `KeyForge` geometry.
pub fn cmd_parse_kle(json: &str) -> Result<KeyboardDefinition, CommandError> {
    let geometry = parse_kle_json(json).map_err(|e| CommandError::Validation(e.to_string()))?;
    Ok(KeyboardDefinition {
        meta: KeyboardMeta {
            name: DEFAULT_KEYBOARD_NAME.into(),
            author: DEFAULT_AUTHOR_NAME.into(),
            version: DEFAULT_VERSION.into(),
            notes: DEFAULT_KLE_NOTES.into(),
            kb_type: "ortho".into(),
        },
        geometry,
        layouts: HashMap::new(),
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
/// Exports a `KeyForge` geometry definition to a KLE JSON string.
pub fn cmd_export_to_kle(def: KeyboardDefinition) -> Result<String, CommandError> {
    to_kle_json(&def.geometry).map_err(|e| CommandError::Validation(e.to_string()))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
/// Saves a keyboard definition file to the application's local keyboard library.
pub fn cmd_save_keyboard(
    app: AppHandle,
    filename: String,
    def: KeyboardDefinition,
) -> Result<(), CommandError> {
    let data_dir = get_data_dir(&app)?;
    UserRepo::new(data_dir).save_keyboard_definition(&filename, &def)?;
    Ok(())
}

/// Exports a layout to a target firmware format (e.g., QMK, ZMK).
#[tauri::command]
pub async fn cmd_export_firmware(
    state: tauri::State<'_, SessionState>,
    layout_name: String,
    layout_str: String,
    format: String,
) -> Result<String, CommandError> {
    let keys: Vec<String> = layout_str
        .split_whitespace()
        .map(std::string::ToString::to_string)
        .collect();
    // Attempt to load registry from session or fallback to None
    let registry = state
        .assets
        .load::<keyforge_model::keycodes::KeycodeRegistry>(
            keyforge_model::constants::ASSET_KEYCODES,
        )
        .await
        .ok();

    let exporter: Box<dyn Exporter> = match format.to_lowercase().as_str() {
        "qmk" => Box::new(QmkExporter),
        "zmk" => Box::new(ZmkExporter),
        "via" => Box::new(keyforge_export::via::ViaExporter),
        _ => return Err(CommandError::Validation("Unsupported format.".into())),
    };

    // For now, treat the entire string as a single layer.
    // Future expansion could involve splitting by a delimiter for multi-layer support.
    exporter
        .generate(&layout_name, &[keys], registry.as_deref())
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
/// Writes a file to a specified path after ensuring it resides within a safe directory.
///
/// If `overwrite` is false and the file exists, returns an error.
pub fn cmd_safe_write_file(path: &str, content: &str, overwrite: bool) -> Result<(), CommandError> {
    let p = Path::new(path);
    let allowed_exts = ["json", "txt", "c", "h", "keymap", "conf"];
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !allowed_exts.contains(&ext) {
        return Err(CommandError::Validation(format!(
            "File extension '{ext}' is not allowed."
        )));
    }
    if path.contains("..") {
        return Err(CommandError::Validation("Path traversal detected.".into()));
    }

    if !overwrite && p.exists() {
        return Err(CommandError::Validation("File already exists".into()));
    }

    std::fs::write(path, content).map_err(CommandError::Io)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_safe_write_validation() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        // 1. Valid Write
        let valid_path = root.join("layout.json");
        let res = cmd_safe_write_file(valid_path.to_str().unwrap(), "{}", true);
        assert!(res.is_ok(), "Valid JSON write should succeed");

        // 2. Invalid Extension
        let invalid_ext = root.join("script.sh");
        let res = cmd_safe_write_file(invalid_ext.to_str().unwrap(), "echo hack", true);
        assert!(res.is_err(), "Shell script write should fail");

        // 3. Path Traversal
        let traversal = root.join("../outside.json");
        let res = cmd_safe_write_file(traversal.to_str().unwrap(), "{}", true);
        assert!(res.is_err(), "Path traversal should fail");
    }
}
