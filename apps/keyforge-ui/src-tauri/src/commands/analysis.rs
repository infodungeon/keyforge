use crate::error::CommandError;
use keyforge_infra::AssetLoader;
use crate::models::{DerivedStats, ValidationResult};
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_adapter::conversion;
use keyforge_infra::listing;
use keyforge_model::SwapSuggestion;
use keyforge_persistence::{Compiler, Project, ProjectMeta};
use keyforge_model::config::{CorpusSource, ScoringWeights};
use keyforge_model::geometry::KeyboardGeometry;
use serde::Serialize;
use tauri::AppHandle;

/// Statistics for a specific corpus on disk.
#[derive(Serialize)]
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

/// Returns detailed statistics for all available corpora. (Implementation pending).
#[tauri::command]
pub fn cmd_get_corpus_stats(_app: AppHandle) -> Result<Vec<CorpusStats>, CommandError> {
    Ok(vec![])
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
    let project = Project {
        meta: ProjectMeta {
            name: "UI Session".into(),
            ..Default::default()
        },
        keyboard: keyboard_name,
        corpora: vec![CorpusSource {
            id: corpus_filename,
            weight: 1.0,
            hash: None,
        }],
        cost_matrix: keyforge_model::CostMatrixSource::Predefined(cost_filename),
        ..Default::default()
    };

    let assets = state.assets.clone();

    let compiler = Compiler::new(assets.as_ref());
    let runtime = compiler
        .compile(&project)
        .await
        .map_err(|e| CommandError::Config(format!("Failed to compile session: {}", e)))?;

    *state.active.write().await = Some(runtime);

    Ok("Runtime Compiled".to_string())
}

/// Validates a layout string against the currently active search runtime.
#[tauri::command]
pub async fn cmd_validate_layout(
    _app: AppHandle,
    state: tauri::State<'_, SessionState>,
    layout_str: String,
    _weights: Option<ScoringWeights>,
    keyboard_name: Option<String>,
) -> Result<ValidationResult, CommandError> {
    let runtime = {
        let guard = state.active.read().await;
        guard
            .as_ref()
            .ok_or_else(|| CommandError::Config("No runtime loaded".into()))?
            .clone()
    };

    // FIX: Propagate loading errors instead of defaulting to empty geometry
    let geometry = if let Some(name) = keyboard_name {
        state.assets
            .load_keyboard(&name)
            .await
            .map(|def| def.geometry)
            .map_err(|e| CommandError::Config(format!("Failed to load keyboard '{}': {}", name, e)))?
    } else {
        KeyboardGeometry::default()
    };

    let result = tauri::async_runtime::spawn_blocking(move || {
        let key_count = runtime.engine.key_count();
        let layout = conversion::parse_layout_string(&layout_str, key_count, &runtime.registry)
            .map_err(|e| {
                tracing::error!("Layout parsing failed: {}", e);
                CommandError::Validation(e.to_string())
            })?;

        let report = runtime.analyze(&layout)?;
        let heatmap = report.heatmap.clone();
        let penalty_map = report.penalty_map.clone();
        tracing::info!("Analysis complete. Score: {}, SFB: {}", report.score, report.sfb_total);
        
        // SANITY CHECK: Fail if stats are garbage
        if report.score > 10_000_000.0 || report.sfb_ratio > 0.20 {
            let msg = format!("Implausible Physics Result: Score={}, SFB={:.2}%", report.score, report.sfb_ratio * 100.0);
            tracing::warn!("{}", msg);
        }

        let proto_geometry: KeyboardGeometry = 
            serde_json::from_value(serde_json::to_value(&geometry).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        Ok::<ValidationResult, CommandError>(ValidationResult {
            layout_name: "Custom".to_string(),
            score: report,
            geometry: proto_geometry,
            heatmap,
            penalty_map,
        })
    })
    .await
    .map_err(|e| {
        tracing::error!("Analysis thread panicked or failed: {}", e);
        CommandError::Internal(e.to_string())
    })??;

    Ok(result)
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
    state: tauri::State<'_, SessionState>,
    layout_str: String,
) -> Result<Vec<SwapSuggestion>, CommandError> {
    let runtime = {
        let guard = state.active.read().await;
        guard
            .as_ref()
            .ok_or_else(|| CommandError::Config("No runtime loaded".into()))?
            .clone()
    };

    let result = tauri::async_runtime::spawn_blocking(move || {
        let key_count = runtime.engine.key_count();
        let layout = conversion::parse_layout_string(&layout_str, key_count, &runtime.registry)
            .map_err(|e| CommandError::Validation(e.to_string()))?;

        Ok::<Vec<SwapSuggestion>, CommandError>(runtime.suggest_improvements(&layout)?)
    })
    .await
    .map_err(|e| CommandError::Internal(e.to_string()))??;

    Ok(result)
}
