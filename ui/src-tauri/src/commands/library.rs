use crate::models::UserLayoutStore;
use crate::utils::{atomic_write, get_data_dir};
use keyforge_core::api::KeyForgeState;
use keyforge_core::geometry::kle::parse_kle_json;
use keyforge_core::geometry::{KeyboardDefinition, KeyboardGeometry, KeyboardMeta};
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use tauri::AppHandle;

#[tauri::command]
pub fn cmd_list_keyboards(app: AppHandle) -> Result<Vec<String>, String> {
    let root = get_data_dir(&app)?;
    let path = root.join("keyboards");

    if !path.exists() {
        return Err(format!("Path not found: {:?}", path));
    }

    let entries = fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                files.push(stem.to_string());
            }
        }
    }
    files.sort();
    Ok(files)
}

#[tauri::command]
pub fn cmd_get_loaded_layouts(
    state: tauri::State<KeyForgeState>,
) -> Result<HashMap<String, String>, String> {
    // FIXED: Uses read() for RwLock
    let sessions = state.sessions.read().map_err(|e| e.to_string())?;
    let session = sessions.get("primary").ok_or("No keyboard loaded")?;
    Ok(session.kb_def.layouts.clone())
}

#[tauri::command]
pub fn cmd_get_keyboard_geometry(app: AppHandle, name: String) -> Result<KeyboardGeometry, String> {
    let root = get_data_dir(&app)?;
    let path = root.join("keyboards").join(format!("{}.json", name));

    if !path.exists() {
        return Err(format!("Keyboard file not found: {:?}", path));
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let def: KeyboardDefinition = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(def.geometry)
}

#[tauri::command]
pub fn cmd_get_all_layouts_scoped(
    app: AppHandle,
    state: tauri::State<KeyForgeState>,
    keyboard_id: String,
) -> Result<HashMap<String, String>, String> {
    // FIXED: Uses read() for RwLock
    let sessions = state.sessions.read().map_err(|e| e.to_string())?;

    let mut all_layouts = if let Some(session) = sessions.get("primary") {
        session.kb_def.layouts.clone()
    } else {
        HashMap::new()
    };

    let data_dir = get_data_dir(&app)?;
    let user_path = data_dir.join("user_layouts.json");

    if user_path.exists() {
        if let Ok(content) = fs::read_to_string(user_path) {
            if let Ok(store) = serde_json::from_str::<UserLayoutStore>(&content) {
                if let Some(kb_layouts) = store.layouts.get(&keyboard_id) {
                    for (name, layout) in kb_layouts {
                        all_layouts.insert(name.clone(), layout.clone());
                    }
                }
            }
        }
    }

    Ok(all_layouts)
}

#[tauri::command]
pub fn cmd_save_user_layout(
    app: AppHandle,
    keyboard_id: String,
    name: String,
    layout: String,
) -> Result<(), String> {
    let data_dir = get_data_dir(&app)?;
    let file_path = data_dir.join("user_layouts.json");

    let mut store = if file_path.exists() {
        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<UserLayoutStore>(&content).unwrap_or_default()
    } else {
        UserLayoutStore::default()
    };

    let kb_entry = store
        .layouts
        .entry(keyboard_id)
        .or_insert_with(HashMap::new);
    kb_entry.insert(name, layout);

    let json = serde_json::to_string_pretty(&store).map_err(|e| e.to_string())?;

    // FIXED: Uses atomic_write
    atomic_write(file_path, json).map_err(|e| format!("Save failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn cmd_delete_user_layout(
    app: AppHandle,
    keyboard_id: String,
    name: String,
) -> Result<(), String> {
    let data_dir = get_data_dir(&app)?;
    let file_path = data_dir.join("user_layouts.json");

    if !file_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let mut store = serde_json::from_str::<UserLayoutStore>(&content).map_err(|e| e.to_string())?;

    if let Some(kb_layouts) = store.layouts.get_mut(&keyboard_id) {
        kb_layouts.remove(&name);
    }

    let json = serde_json::to_string_pretty(&store).map_err(|e| e.to_string())?;

    // FIXED: Uses atomic_write
    atomic_write(file_path, json).map_err(|e| format!("Delete failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn cmd_submit_user_layout(
    hive_url: String,
    name: String,
    layout: String,
    author: String,
) -> Result<String, String> {
    let client = Client::new();
    let res = client
        .post(format!("{}/submissions", hive_url))
        .json(&serde_json::json!({
            "name": name,
            "layout": layout,
            "author": author
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok("Submitted".to_string())
    } else {
        Err(format!("Server Error: {}", res.status()))
    }
}

#[tauri::command]
pub fn cmd_parse_kle(json: String) -> Result<KeyboardDefinition, String> {
    let geometry = parse_kle_json(&json).map_err(|e| e.to_string())?;
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
pub fn cmd_save_keyboard(
    app: AppHandle,
    filename: String,
    def: KeyboardDefinition,
) -> Result<(), String> {
    let data_dir = get_data_dir(&app)?;
    let kb_dir = data_dir.join("keyboards");
    if !kb_dir.exists() {
        fs::create_dir_all(&kb_dir).map_err(|e| e.to_string())?;
    }
    let safe_name = filename.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "");
    let path = kb_dir.join(format!("{}.json", safe_name));
    let json = serde_json::to_string_pretty(&def).map_err(|e| e.to_string())?;

    // FIXED: Uses atomic_write
    atomic_write(path, json).map_err(|e| format!("Save failed: {}", e))?;

    Ok(())
}
