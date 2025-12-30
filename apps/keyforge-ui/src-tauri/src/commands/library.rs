use crate::error::CommandError;
use keyforge_infra::AssetLoader;
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_export::{qmk::QmkExporter, zmk::ZmkExporter, Exporter};
use keyforge_infra::listing;
use keyforge_infra::HiveClient;
use keyforge_infra::UserRepo;
use keyforge_model::geometry::kle::{parse_kle_json, to_kle_json};
use keyforge_model::geometry::{KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use std::collections::HashMap;
use std::path::Path;
use tauri::AppHandle;

#[tauri::command]
pub fn cmd_list_keyboards(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app).map_err(CommandError::Config)?;
    listing::list_keyboards(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub fn cmd_list_keymap_extras(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app).map_err(CommandError::Config)?;
    listing::list_keymap_extras(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn cmd_get_loaded_layouts(
    _state: tauri::State<'_, SessionState>,
) -> Result<HashMap<String, String>, CommandError> {
    Ok(HashMap::new())
}

#[tauri::command]
pub async fn cmd_get_keyboard_geometry(
    _app: AppHandle,
    state: tauri::State<'_, SessionState>,
    name: String,
) -> Result<KeyboardGeometry, CommandError> {
    state
        .assets
        .load_keyboard(&name)
        .await
        .map(|def| def.geometry)
        .map_err(|e| CommandError::Config(format!("Failed to load geometry: {}", e)))
}

#[tauri::command]
pub async fn cmd_get_all_layouts_scoped(
    app: AppHandle,
    state: tauri::State<'_, SessionState>,
    keyboard_id: String,
) -> Result<HashMap<String, String>, CommandError> {
    let data_dir = get_data_dir(&app).map_err(CommandError::Config)?;
    let user_data = UserRepo::new(data_dir);
    let mut all_layouts = user_data.get_layouts(&keyboard_id);

    if let Ok(def) = state.assets.load_keyboard(&keyboard_id).await {
        all_layouts.extend(def.layouts);
    }
    Ok(all_layouts)
}

#[tauri::command]
pub fn cmd_save_user_layout(
    app: AppHandle,
    keyboard_id: String,
    name: String,
    layout: String,
) -> Result<(), CommandError> {
    let data_dir = get_data_dir(&app).map_err(CommandError::Config)?;
    UserRepo::new(data_dir)
        .save_layout(&keyboard_id, &name, &layout)
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub fn cmd_delete_user_layout(
    app: AppHandle,
    keyboard_id: String,
    name: String,
) -> Result<(), CommandError> {
    let data_dir = get_data_dir(&app).map_err(CommandError::Config)?;
    UserRepo::new(data_dir)
        .delete_layout(&keyboard_id, &name)
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn cmd_submit_user_layout(
    hive_url: String,
    hive_secret: String,
    name: String,
    layout: String,
    author: String,
) -> Result<String, CommandError> {
    let client = HiveClient::new(hive_url, Some(hive_secret))
        .map_err(|e| CommandError::Config(e.to_string()))?;
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
pub fn cmd_parse_kle(json: String) -> Result<KeyboardDefinition, CommandError> {
    let geometry = parse_kle_json(&json).map_err(|e| CommandError::Validation(e.to_string()))?;
    Ok(KeyboardDefinition {
        meta: KeyboardMeta {
            name: "Untitled Board".into(),
            author: "Unknown".into(),
            version: "1.0".into(),
            notes: "Imported from KLE".into(),
            kb_type: "ortho".into(),
        },
        geometry,
        layouts: HashMap::new(),
    })
}

#[tauri::command]
pub fn cmd_export_to_kle(def: KeyboardDefinition) -> Result<String, CommandError> {
    to_kle_json(&def.geometry).map_err(|e| CommandError::Validation(e.to_string()))
}

#[tauri::command]
pub fn cmd_save_keyboard(
    app: AppHandle,
    filename: String,
    def: KeyboardDefinition,
) -> Result<(), CommandError> {
    let data_dir = get_data_dir(&app).map_err(CommandError::Config)?;
    UserRepo::new(data_dir)
        .save_keyboard_definition(&filename, &def)
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub fn cmd_export_firmware(
    layout_name: String,
    layout_str: String,
    format: String,
) -> Result<String, CommandError> {
    let keys: Vec<String> = layout_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let exporter: Box<dyn Exporter> = match format.to_lowercase().as_str() {
        "qmk" => Box::new(QmkExporter),
        "zmk" => Box::new(ZmkExporter),
        _ => return Err(CommandError::Validation("Unsupported format.".into())),
    };
    exporter
        .generate(&layout_name, &keys)
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub fn cmd_safe_write_file(path: String, content: String) -> Result<(), CommandError> {
    let p = Path::new(&path);
    let allowed_exts = ["json", "txt", "c", "h", "keymap", "conf"];
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !allowed_exts.contains(&ext) {
        return Err(CommandError::Validation(format!(
            "File extension '{}' is not allowed.",
            ext
        )));
    }
    if path.contains("..") {
        return Err(CommandError::Validation("Path traversal detected.".into()));
    }
    std::fs::write(&path, content).map_err(CommandError::Io)
}
