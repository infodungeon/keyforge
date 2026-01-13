use crate::error::CommandError;
use crate::models::{DerivedStats, ValidationResult};
use crate::state::SessionState;
use crate::utils::get_data_dir;
use crate::runner::AgentRunner;
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
    let definition = assets.load_keyboard(&keyboard_name).await
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

    Ok("Dataset Loaded".to_string())
}

/// Validates a layout string against the currently active search runtime.
#[tauri::command]
pub async fn cmd_validate_layout(
    app: AppHandle,
    state: tauri::State<'_, SessionState>,
    layout_str: String,
    _weights: Option<ScoringWeights>,
    _keyboard_name: Option<String>,
) -> Result<ValidationResult, CommandError> {
    let job_config = {
        let guard = state.active_job.read().await;
        guard.as_ref().ok_or(CommandError::Config("No dataset loaded".into()))?.clone()
    };
    
    // Create Runner
    let runner = AgentRunner::new(app.clone());

    // Run
    let json_output = runner.run_validation(&job_config, &layout_str).await?;
    
    // Deserialize report
    // We assume the agent returns a JSON that matches keyforge_model::AnalysisReport
    let report: keyforge_model::AnalysisReport = serde_json::from_str(&json_output)
        .map_err(|e| CommandError::Internal(format!("Failed to parse agent output: {}. Output was: {}", e, json_output)))?;
        
    // Construct ValidationResult
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

/// Suggests a set of character swaps to improve the current layout.
#[tauri::command]
pub async fn cmd_get_smart_swaps(
    _state: tauri::State<'_, SessionState>,
    _layout_str: String,
) -> Result<Vec<SwapSuggestion>, CommandError> {
    // Stubbed: Agent does not yet support swapping
    Ok(vec![])
}
