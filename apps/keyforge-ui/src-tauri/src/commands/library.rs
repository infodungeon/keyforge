// apps/keyforge-ui/src-tauri/src/commands/library.rs

use crate::error::CommandError;
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_infra::fs::listing;
use keyforge_protocol::KeyboardGeometryDto;
use std::sync::Arc;
use tauri::AppHandle;

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_list_keyboards(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app)?;
    listing::list_keyboards(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_list_keymap_extras(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app)?;
    listing::list_keymap_extras(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn cmd_get_loaded_layouts(
    state: tauri::State<'_, Arc<SessionState>>,
) -> Result<Vec<String>, CommandError> {
    let job = state.active_job.read().await;
    Ok(job
        .as_ref()
        .map(|j| j.definition.layouts.keys().cloned().collect())
        .unwrap_or_default())
}

#[tauri::command]
pub async fn cmd_get_all_layouts_scoped(
    _state: tauri::State<'_, Arc<SessionState>>,
) -> Result<Vec<String>, CommandError> {
    Ok(vec![]) // Placeholder
}

#[tauri::command]
pub async fn cmd_get_keyboard_geometry(
    state: tauri::State<'_, Arc<SessionState>>,
) -> Result<KeyboardGeometryDto, CommandError> {
    let job = state.active_job.read().await;
    job.as_ref()
        .map(|j| j.definition.geometry.clone())
        .ok_or_else(|| CommandError::Internal("No active job".into()))
}

#[tauri::command]
pub async fn cmd_save_user_layout(
    _state: tauri::State<'_, Arc<SessionState>>,
    _name: String,
    _layout: String,
) -> Result<(), CommandError> {
    Ok(()) // Placeholder
}

#[tauri::command]
pub async fn cmd_delete_user_layout(
    _state: tauri::State<'_, Arc<SessionState>>,
    _name: String,
) -> Result<(), CommandError> {
    Ok(()) // Placeholder
}

#[tauri::command]
pub async fn cmd_submit_user_layout(
    _state: tauri::State<'_, Arc<SessionState>>,
    _layout: String,
) -> Result<(), CommandError> {
    Ok(()) // Placeholder
}

#[tauri::command]
pub async fn cmd_parse_kle(_json: String) -> Result<KeyboardGeometryDto, CommandError> {
    Err(CommandError::Internal("Not implemented".into()))
}

#[tauri::command]
pub async fn cmd_export_to_kle(
    _state: tauri::State<'_, Arc<SessionState>>,
    _layout: String,
) -> Result<String, CommandError> {
    Ok(String::new()) // Placeholder
}

#[tauri::command]
pub async fn cmd_save_keyboard(
    _state: tauri::State<'_, Arc<SessionState>>,
    _name: String,
    _json: String,
) -> Result<(), CommandError> {
    Ok(()) // Placeholder
}

#[tauri::command]
pub async fn cmd_export_firmware(
    _state: tauri::State<'_, Arc<SessionState>>,
    _layout: String,
    _kb_id: String,
) -> Result<Vec<u8>, CommandError> {
    Ok(vec![]) // Placeholder
}

#[tauri::command]
pub async fn cmd_safe_write_file(_path: String, _content: String) -> Result<(), CommandError> {
    Ok(()) // Placeholder
}
