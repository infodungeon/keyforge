// ===== keyforge/ui/src-tauri/src/commands/analysis.rs =====
use crate::utils::get_data_dir;
use keyforge_core::api::{load_dataset, validate_layout, KeyForgeState};
use keyforge_core::config::ScoringWeights;
use keyforge_core::corpus::generate_ngrams;
use std::fs;
use tauri::AppHandle;

#[tauri::command]
pub fn cmd_list_corpora(app: AppHandle) -> Result<Vec<String>, String> {
    let root = get_data_dir(&app)?;
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "tsv" {
                    if let Some(stem) = path.file_name().and_then(|s| s.to_str()) {
                        files.push(stem.to_string()); // Just the name "ngrams-all.tsv"
                    }
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

// NEW: List available cost matrices
#[tauri::command]
pub fn cmd_list_cost_matrices(app: AppHandle) -> Result<Vec<String>, String> {
    let root = get_data_dir(&app)?;
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "csv" {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        files.push(name.to_string());
                    }
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

#[tauri::command]
pub fn cmd_import_corpus(
    app: AppHandle,
    file_path: String,
    name: String,
) -> Result<String, String> {
    let data_dir = get_data_dir(&app)?;
    let target_path = data_dir.join(format!("{}.tsv", name));

    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read source file: {}", e))?;

    let tsv_content = generate_ngrams(&content, 3000);

    fs::write(&target_path, tsv_content).map_err(|e| format!("Failed to write TSV: {}", e))?;

    Ok(format!("Imported corpus '{}' successfully.", name))
}

#[tauri::command]
pub fn cmd_load_dataset(
    app: AppHandle,
    state: tauri::State<KeyForgeState>,
    keyboard_name: String,
    corpus_filename: String,
    cost_filename: String, // UPDATED: Now accepts variable cost matrix
) -> Result<String, String> {
    let root = get_data_dir(&app)?;

    let cost = root.join(&cost_filename);
    let ngrams = root.join(&corpus_filename);
    let geo = root
        .join("keyboards")
        .join(format!("{}.json", keyboard_name));

    if !cost.exists() {
        return Err(format!("Cost matrix not found: {:?}", cost));
    }
    if !ngrams.exists() {
        return Err(format!("Corpus not found: {:?}", ngrams));
    }
    if !geo.exists() {
        return Err(format!("Keyboard not found: {:?}", geo));
    }

    tracing::info!(
        "Loading Dataset: KB='{}' Corpus='{}' Costs='{}'",
        keyboard_name,
        corpus_filename,
        cost_filename
    );

    load_dataset(
        &state,
        "primary",
        cost.to_str().unwrap(),
        ngrams.to_str().unwrap(),
        &Some(geo.to_str().unwrap().to_string()),
        None,
        Some(root.to_str().unwrap()),
    )
}

#[tauri::command]
pub fn cmd_validate_layout(
    state: tauri::State<KeyForgeState>,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<keyforge_core::api::ValidationResult, String> {
    validate_layout(&state, "primary", layout_str, weights)
}
