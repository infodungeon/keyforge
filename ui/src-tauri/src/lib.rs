use keyforge::api::{load_dataset, validate_layout, KeyForgeState};
use keyforge::config::ScoringWeights;

// Wrapper command to initialize the engine
#[tauri::command]
fn cmd_load_dataset(
    state: tauri::State<KeyForgeState>,
    base_path: String,
) -> Result<String, String> {
    let cost = format!("{}/data/cost_matrix.csv", base_path);
    let ngrams = format!("{}/data/ngrams-all.tsv", base_path);
    let geo = format!("{}/data/szr35.json", base_path);

    println!("Attempting to load from Base Path: {}", base_path);

    // Pass None for corpus_scale to use the default 200M/1B (whatever is in config)
    load_dataset(&state, &cost, &ngrams, &Some(geo), None)
}

#[tauri::command]
fn cmd_validate_layout(
    state: tauri::State<KeyForgeState>,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<keyforge::api::ValidationResult, String> {
    validate_layout(&state, layout_str, weights)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(KeyForgeState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            cmd_load_dataset,
            cmd_validate_layout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
