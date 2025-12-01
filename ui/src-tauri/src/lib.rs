use keyforge::api::{load_dataset, validate_layout, KeyForgeState};
use keyforge::config::{Config, ScoringWeights, SearchParams};
use keyforge::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Window};

// --- Helper Structs ---

// Shared state to allow stopping the search
struct SearchState {
    stop_flag: Arc<Mutex<bool>>,
}

// Data sent to the frontend during optimization
#[derive(Clone, serde::Serialize)]
struct SearchUpdate {
    epoch: usize,
    score: f32,
    layout: String,
    ips: f32,
}

// Request object for starting search with full config
#[derive(serde::Deserialize)]
struct StartSearchRequest {
    pinned_keys: String,
    search_params: SearchParams,
    weights: ScoringWeights,
}

// Callback bridge: Bridges Rust Traits -> Tauri Events
struct TauriBridge {
    window: Window,
    stop_signal: Arc<Mutex<bool>>,
}

impl ProgressCallback for TauriBridge {
    fn on_progress(&self, epoch: usize, score: f32, best_layout: &[u8], ips: f32) -> bool {
        // Check if user clicked "Stop"
        if *self.stop_signal.lock().unwrap() {
            return false; // Abort optimization
        }

        let layout_str = String::from_utf8_lossy(best_layout).to_string();

        // Emit event to React
        let _ = self.window.emit(
            "search-update",
            SearchUpdate {
                epoch,
                score,
                layout: layout_str,
                ips,
            },
        );

        true // Continue
    }
}

// --- Tauri Commands ---

#[tauri::command]
fn cmd_list_keyboards(base_path: String) -> Result<Vec<String>, String> {
    let path = format!("{}/data/keyboards", base_path);
    let entries = fs::read_dir(&path).map_err(|e| e.to_string())?;

    let mut files = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    files.push(stem.to_string());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

#[tauri::command]
fn cmd_get_loaded_layouts(
    state: tauri::State<KeyForgeState>,
) -> Result<HashMap<String, String>, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get("primary").ok_or("No keyboard loaded")?;

    // Return the map of Layout Name -> Layout String
    Ok(session.kb_def.layouts.clone())
}

#[tauri::command]
fn cmd_load_dataset(
    state: tauri::State<KeyForgeState>,
    base_path: String,
    keyboard_name: String,
) -> Result<String, String> {
    let cost = format!("{}/data/cost_matrix.csv", base_path);
    let ngrams = format!("{}/data/ngrams-all.tsv", base_path);
    let geo = format!("{}/data/keyboards/{}.json", base_path, keyboard_name);

    tracing::info!("Loading Keyboard: {}", keyboard_name);

    // UPDATED: Pass Some(&base_path) as the last argument
    load_dataset(
        &state,
        "primary",
        &cost,
        &ngrams,
        &Some(geo),
        None,
        Some(&base_path),
    )
}

#[tauri::command]
fn cmd_validate_layout(
    state: tauri::State<KeyForgeState>,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<keyforge::api::ValidationResult, String> {
    validate_layout(&state, "primary", layout_str, weights)
}

#[tauri::command]
async fn cmd_start_search(
    window: Window,
    state: tauri::State<'_, KeyForgeState>,
    search_state: tauri::State<'_, SearchState>,
    request: StartSearchRequest,
) -> Result<String, String> {
    let scorer_arc = {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        let session = sessions.get("primary").ok_or("Session not loaded")?;

        // Clone scorer and apply new weights from UI
        let mut scorer = session.scorer.clone();
        scorer.weights = request.weights;
        Arc::new(scorer)
    };

    let mut config = Config::default();
    config.search = request.search_params;
    config.search.pinned_keys = request.pinned_keys;

    let options = OptimizationOptions::from(&config);
    let optimizer = Optimizer::new(scorer_arc, options);

    // Reset Stop Signal
    *search_state.stop_flag.lock().unwrap() = false;

    let bridge = TauriBridge {
        window,
        stop_signal: search_state.stop_flag.clone(),
    };

    tracing::info!("Starting Deep Optimization...");

    // Run on a blocking thread to avoid freezing the async runtime
    let result = tauri::async_runtime::spawn_blocking(move || optimizer.run(None, bridge))
        .await
        .map_err(|e| e.to_string())?;

    Ok(String::from_utf8_lossy(&result.layout_bytes).to_string())
}

#[tauri::command]
fn cmd_stop_search(search_state: tauri::State<SearchState>) {
    *search_state.stop_flag.lock().unwrap() = true;
}

// --- Entry Point ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(KeyForgeState::default())
        .manage(SearchState {
            stop_flag: Arc::new(Mutex::new(false)),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            cmd_list_keyboards,
            cmd_get_loaded_layouts,
            cmd_load_dataset,
            cmd_validate_layout,
            cmd_start_search,
            cmd_stop_search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
