use crate::error::CommandError;
use crate::models::{DerivedStats, ValidationResult};
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_infra::AssetLoader;
use keyforge_infra::listing;
use keyforge_model::SwapSuggestion;
use keyforge_model::config::ScoringWeights;
use keyforge_protocol::JobConfig;
use serde::Serialize;
use tauri::AppHandle;

/// Statistics for a specific corpus on disk.
#[derive(Serialize, Debug)]
pub struct CorpusStats {
    /// Name of the corpus.
    pub name: String,
    /// Size of the processed corpus data in bytes.
    pub size_bytes: u64,
    /// Canonical filesystem path to the corpus file.
    pub path: String,
}

/// Lists all available corpora in the application's data directory.
#[tauri::command]
pub fn cmd_list_corpora(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app).map_err(CommandError::Config)?;
    listing::list_corpora(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

/// Returns detailed statistics for all available corpora.
#[tauri::command]
pub fn cmd_get_corpus_stats(app: AppHandle) -> Result<Vec<CorpusStats>, CommandError> {
    let root = get_data_dir(&app).map_err(CommandError::Config)?;
    let ids = listing::list_corpora(&root).map_err(|e| CommandError::Internal(e.to_string()))?;
    
    let mut stats = Vec::new();
    for id in ids {
        // Try system then user
        let sys_path = root.join("system/corpora").join(&id).join("1grams.mpk.zst");
        let usr_path = root.join("user/corpora").join(&id).join("1grams.json");
        
        let (path, size) = if sys_path.exists() {
            (sys_path.to_string_lossy().to_string(), std::fs::metadata(&sys_path).map(|m| m.len()).unwrap_or(0))
        } else if usr_path.exists() {
            (usr_path.to_string_lossy().to_string(), std::fs::metadata(&usr_path).map(|m| m.len()).unwrap_or(0))
        } else {
            continue;
        };

        stats.push(CorpusStats {
            name: id,
            size_bytes: size,
            path,
        });
    }
    Ok(stats)
}

/// Lists all available cost matrices in the application's data directory.
#[tauri::command]
pub fn cmd_list_cost_matrices(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let root = get_data_dir(&app).map_err(CommandError::Config)?;
    listing::list_cost_matrices(&root).map_err(|e| CommandError::Internal(e.to_string()))
}

/// Compiles a new search runtime using the specified assets and stores it in the session state.
#[tauri::command]
pub async fn cmd_load_dataset(
    _app: AppHandle,
    state: tauri::State<'_, SessionState>,
    keyboard_name: String,
    corpus_filename: String,
    cost_filename: String,
    _extras: Vec<String>,
) -> Result<String, CommandError> {
    let assets = &state.assets;
    
    // Load definition to ensure it exists and to put in config
    let definition = assets.load::<keyforge_model::KeyboardDefinition>(&keyboard_name).await
        .map_err(|e| CommandError::Config(format!("Failed to load keyboard: {}", e)))?;
    
    let job_config = JobConfig {
        definition: definition.as_ref().clone(),
        weights: keyforge_model::config::ScoringWeights::default(),
        params: keyforge_model::config::SearchParams::default(),
        pinned_keys: vec![],
        corpora: vec![keyforge_model::config::CorpusSource {
            id: corpus_filename,
            weight: 1.0,
            hash: None,
        }],
        cost_matrix: keyforge_model::CostMatrixSource::Predefined(cost_filename),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    *state.active_job.write().await = Some(job_config);
    *state.scoring_session.write().await = None;

    Ok("Dataset Loaded".to_string())
}

use keyforge_runner::{OptimizationRunner, RunnerOptions};

/// Validates a layout string against the currently active search runtime.
#[tauri::command]
pub async fn cmd_validate_layout(
    _app: AppHandle,
    state: tauri::State<'_, SessionState>,
    layout_str: String,
    _weights: Option<ScoringWeights>,
    _keyboard_name: Option<String>,
) -> Result<ValidationResult, CommandError> {
    let job_config = {
        let guard = state.active_job.read().await;
        guard.as_ref().ok_or(CommandError::Config("No dataset loaded".into()))?.clone()
    };
    
    // 1. Ensure Session is Cached (Double-Checked Locking)
    {
        let session_guard = state.scoring_session.read().await;
        if session_guard.is_none() {
            drop(session_guard);
            let mut write_guard = state.scoring_session.write().await;
            if write_guard.is_none() {
                let options = RunnerOptions {
                    keycodes_file: "keycodes.json".to_string(),
                     ..Default::default()
                };

                let session = OptimizationRunner::prepare_session(
                    state.assets.as_ref(),
                    &job_config,
                    &options
                ).await.map_err(|e| CommandError::Internal(format!("Failed to compile engine: {}", e)))?;
                
                *write_guard = Some(session);
            }
        }
    }

    // 2. Use Session
    let session_guard = state.scoring_session.read().await;
    let session = session_guard.as_ref().ok_or(CommandError::Internal("Session lost".into()))?;

    // 3. Parse Layout
    let layout_parsed = keyforge_adapter::conversion::parse_layout_string(
        &layout_str,
        session.engine.key_count(),
        &session.registry
    ).map_err(|e| CommandError::Internal(format!("Invalid layout string: {}", e)))?;

    // 4. Analyze
    let report = session.engine.analyze(&layout_parsed)
        .map_err(|e| CommandError::Internal(format!("Analysis failed: {:?}", e)))?;
    
    Ok(ValidationResult {
        layout_name: "Custom".to_string(),
        score: report.clone(),
        geometry: job_config.definition.geometry.clone(),
        heatmap: report.heatmap,
        penalty_map: report.penalty_map,
    })
}

/// Returns derived statistics for a layout, such as hand balance.
#[tauri::command]
pub async fn cmd_get_layout_stats(
    _state: tauri::State<'_, SessionState>,
    _layout_str: String,
) -> Result<DerivedStats, CommandError> {
    Ok(DerivedStats { hand_balance: 0.0 })
}

#[tauri::command]
pub async fn cmd_get_smart_swaps(
    state: tauri::State<'_, SessionState>,
    layout_str: String,
    include_thumbs: Option<bool>,
) -> Result<Vec<SwapSuggestion>, CommandError> {
    // 1. Get Session
    let session_guard = state.scoring_session.read().await;
    let session = session_guard.as_ref().ok_or(CommandError::Internal("Session lost".into()))?;

    // 2. Parse Layout
    let layout_parsed = keyforge_adapter::conversion::parse_layout_string(
        &layout_str,
        session.engine.key_count(),
        &session.registry
    ).map_err(|e| CommandError::Internal(format!("Invalid layout string: {}", e)))?;

    // 3. Get Suggestions
    let suggestions = session.engine.suggest_improvements(&layout_parsed, include_thumbs.unwrap_or(false));
    
    Ok(suggestions)
}
