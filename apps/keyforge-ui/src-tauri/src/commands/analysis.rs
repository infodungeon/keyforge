// apps/keyforge-ui/src-tauri/src/commands/analysis.rs

use crate::error::CommandError;
use crate::models::{AnalysisReportDto, DerivedStats, SwapSuggestionDto, ValidationResult};
use crate::state::SessionState;
use crate::utils::get_data_dir;
use keyforge_adapter::loader::AssetLoader;
use keyforge_infra::fs::listing;
use keyforge_model::KeyboardDefinition;
use keyforge_protocol::{CorpusSourceDto, CostMatrixSourceDto, JobConfig};
use serde::Serialize;
use tauri::AppHandle;

/// Statistics for a specific corpus on disk.
#[derive(Serialize, Debug)]
pub struct CorpusStats {
    pub name: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub async fn cmd_list_corpora(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let data_dir = get_data_dir(&app)?;
    let corpora_dir = data_dir.join("corpora");
    if !corpora_dir.exists() {
        return Ok(vec![]);
    }
    listing::list_files(&corpora_dir, &["json".to_string()])
        .map(|paths| {
            paths
                .into_iter()
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .collect()
        })
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn cmd_get_corpus_stats(
    app: AppHandle,
    name: String,
) -> Result<CorpusStats, CommandError> {
    let data_dir = get_data_dir(&app)?;
    let path = data_dir.join("corpora").join(&name);
    let meta = std::fs::metadata(&path).map_err(|e| CommandError::Internal(e.to_string()))?;
    Ok(CorpusStats {
        name,
        size_bytes: meta.len(),
    })
}

#[tauri::command]
pub async fn cmd_list_cost_matrices(app: AppHandle) -> Result<Vec<String>, CommandError> {
    let data_dir = get_data_dir(&app)?;
    let weights_dir = data_dir.join("weights");
    if !weights_dir.exists() {
        return Ok(vec![]);
    }
    listing::list_files(&weights_dir, &["json".to_string()])
        .map(|paths| {
            paths
                .into_iter()
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .collect()
        })
        .map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn cmd_load_dataset(_app: AppHandle, _name: String) -> Result<(), CommandError> {
    Ok(()) // Placeholder
}

#[tauri::command]
pub fn get_available_corpora(app: &AppHandle) -> Result<Vec<CorpusStats>, CommandError> {
    let data_dir = get_data_dir(app)?;
    let corpora_dir = data_dir.join("corpora");

    if !corpora_dir.exists() {
        return Ok(vec![]);
    }

    let files = listing::list_files(&corpora_dir, &["json".to_string()])
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    let mut stats = vec![];
    for file in files {
        let meta = std::fs::metadata(&file).map_err(|e| CommandError::Internal(e.to_string()))?;
        stats.push(CorpusStats {
            name: file.file_name().map_or_else(
                || "unknown".to_string(),
                |n| n.to_string_lossy().to_string(),
            ),
            size_bytes: meta.len(),
        });
    }

    Ok(stats)
}

#[tauri::command]
pub async fn cmd_validate_layout(
    state: tauri::State<'_, std::sync::Arc<SessionState>>,
    layout_str: String,
) -> Result<AnalysisReportDto, CommandError> {
    let read_guard = state.scoring_session.read().await;
    let session = read_guard
        .as_ref()
        .ok_or_else(|| CommandError::Internal("No active session".into()))?;

    let layout = keyforge_adapter::conversion::parse_layout_string(
        &layout_str,
        session.engine.key_count(),
        &session.registry,
    )
    .map_err(|e| CommandError::Internal(e.to_string()))?;

    let report = session
        .engine
        .analyze(&layout)
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    Ok(report.into())
}

#[tauri::command]
pub async fn cmd_get_layout_stats(
    state: tauri::State<'_, std::sync::Arc<SessionState>>,
    layout_str: String,
) -> Result<AnalysisReportDto, CommandError> {
    cmd_validate_layout(state, layout_str).await
}

#[tauri::command]
pub async fn cmd_get_smart_swaps(
    _state: tauri::State<'_, std::sync::Arc<SessionState>>,
    _layout_str: String,
) -> Result<Vec<SwapSuggestionDto>, CommandError> {
    // suggest_swaps moved to a separate suggested implementation or different trait
    // For now returning empty to clear compilation
    Ok(vec![])
}

#[tauri::command]
pub async fn validate_layout_string(
    state: tauri::State<'_, std::sync::Arc<SessionState>>,
    layout_str: String,
    keyboard_filename: String,
    corpus_filename: String,
) -> Result<ValidationResult, CommandError> {
    // 1. Resolve Keyboard Definition
    let definition = state
        .assets
        .load::<KeyboardDefinition>(&keyboard_filename)
        .await
        .map_err(|_| CommandError::NotFound)?;

    // 2. Prepare analysis request
    let job_config = JobConfig {
        definition: (*definition).clone().into(),
        weights: keyforge_model::config::ScoringWeights::default().into(),
        params: keyforge_model::config::SearchParams::default().into(),
        pinned_keys: vec![].into(),
        corpora: vec![CorpusSourceDto {
            id: corpus_filename,
            weight: 1.0,
            hash: None,
        }]
        .into(),
        cost_matrix: CostMatrixSourceDto::Predefined("default".to_string()),
        biometrics: vec![].into(),
        parent_job_id: None,
        baseline_score: None,
        parents: vec![].into(),
    };

    // 3. Perform Analysis via shared session or new one
    let report = {
        let read_guard = state.scoring_session.read().await;
        if let Some(session) = &*read_guard {
            let layout = keyforge_adapter::conversion::parse_layout_string(
                &layout_str,
                session.engine.key_count(),
                &session.registry,
            )
            .map_err(|e| CommandError::Internal(e.to_string()))?;
            session
                .engine
                .analyze(&layout)
                .map_err(|e| CommandError::Internal(e.to_string()))?
        } else {
            drop(read_guard);
            let mut write_guard = state.scoring_session.write().await;
            if write_guard.is_none() {
                let builder = keyforge_compute::SessionBuilder::new(state.assets.as_ref())
                    .with_keyboard_def(std::sync::Arc::new(KeyboardDefinition::from_geometry(
                        job_config.to_domain_geometry(),
                        "ui",
                    )))
                    .with_corpus(&job_config.to_domain_corpus_sources())
                    .await?
                    .with_cost_matrix(&job_config.to_domain_cost_matrix())
                    .await?
                    .with_keycodes("default")
                    .await?
                    .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
                        &job_config.to_domain_weights(),
                    ));

                let session = builder.build()?;
                *write_guard = Some(session);
            }
            let session = write_guard
                .as_ref()
                .ok_or_else(|| CommandError::Internal("Failed to initialize session".into()))?;
            let layout = keyforge_adapter::conversion::parse_layout_string(
                &layout_str,
                session.engine.key_count(),
                &session.registry,
            )
            .map_err(|e| CommandError::Internal(e.to_string()))?;
            session
                .engine
                .analyze(&layout)
                .map_err(|e| CommandError::Internal(e.to_string()))?
        }
    };

    Ok(ValidationResult {
        layout_name: "Custom".to_string(),
        score: report.clone().into(),
        geometry: job_config
            .to_domain_geometry()
            .keys
            .into_iter()
            .map(Into::into)
            .collect(),
        heatmap: report.heatmap,
        penalty_map: report.penalty_map,
    })
}

#[tauri::command]
pub async fn get_derived_stats(
    state: tauri::State<'_, std::sync::Arc<SessionState>>,
    layout_str: String,
) -> Result<DerivedStats, CommandError> {
    let read_guard = state.scoring_session.read().await;
    let session = read_guard
        .as_ref()
        .ok_or_else(|| CommandError::Internal("No active session".into()))?;

    let layout = keyforge_adapter::conversion::parse_layout_string(
        &layout_str,
        session.engine.key_count(),
        &session.registry,
    )
    .map_err(|e| CommandError::Internal(e.to_string()))?;

    let report = session
        .engine
        .analyze(&layout)
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    Ok(DerivedStats {
        hand_balance: report.hand_balance,
    })
}
