// libs/keyforge-evolution/src/supervisor/optimizer.rs

use super::strategies::{CoolingAnnealing, GroupMutation};
use super::traits::RealTimeKeeper;
use super::AnnealingConfig;
use crate::errors::EvolutionError;
use crate::supervisor::annealing::Optimizer;
use crate::{NoOpCallback, ProgressCallback};
use keyforge_model::{KeyCode, Layout, OptimizationResult, SearchConfig};
use keyforge_physics::{EngineRequest, ScoringEngine};
use std::sync::Arc;

/// Performs a basic optimization run.
///
/// # Errors
///
/// Returns `EvolutionError::Config` if the request is invalid.
pub fn optimize(req: &EngineRequest) -> Result<OptimizationResult, EvolutionError> {
    optimize_with_callback(req, NoOpCallback)
}

/// Optimizes a keyboard layout with a progress callback.
///
/// # Errors
///
/// Returns `EvolutionError::Config` if the request is invalid.
pub fn optimize_with_callback<CB: ProgressCallback>(
    req: &EngineRequest,
    callback: CB,
) -> Result<OptimizationResult, EvolutionError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let engine_arc = Arc::new(engine);

    // Determine unlocked indices
    let unlocked_indices: Vec<usize> = (0..engine_arc.key_count())
        .filter(|&i| i >= req.pinned_keys.len() || req.pinned_keys[i].is_none())
        .collect();

    evolve_internal(
        &engine_arc,
        &req.config,
        unlocked_indices,
        req.initial_layout.clone(),
        callback,
        Some(&req.pinned_keys),
    )
}

/// Performs optimization using a pre-compiled `ScoringEngine`.
///
/// # Errors
///
/// Returns `EvolutionError::Config` if the search parameters are inconsistent.
pub fn evolve<CB: ProgressCallback>(
    engine: &Arc<ScoringEngine>,
    config: &SearchConfig,
    callback: CB,
    initial_layout: Option<Layout>,
    pinned_keys: Option<&[Option<KeyCode>]>,
) -> Result<OptimizationResult, EvolutionError> {
    // Determine unlocked indices
    let unlocked_indices: Vec<usize> = (0..engine.key_count())
        .filter(|&i| match pinned_keys {
            Some(pins) => i >= pins.len() || pins[i].is_none(),
            None => true,
        })
        .collect();

    evolve_internal(
        engine,
        config,
        unlocked_indices,
        initial_layout,
        callback,
        pinned_keys,
    )
}

/// Internal helper to share logic between legacy and new entry points.
fn evolve_internal<CB: ProgressCallback>(
    engine: &Arc<ScoringEngine>,
    config: &SearchConfig,
    unlocked_indices: Vec<usize>,
    initial_layout: Option<Layout>,
    callback: CB,
    pinned_keys: Option<&[Option<KeyCode>]>,
) -> Result<OptimizationResult, EvolutionError> {
    let mut layout = initial_layout.unwrap_or_else(|| {
        #[allow(clippy::cast_possible_truncation)]
        let keys: Vec<KeyCode> = (0..engine.key_count()).map(|i| KeyCode(i as u16)).collect();
        Layout::new_unchecked(keys)
    });

    // Guardrail: Ensure layout matches engine geometry
    if layout.keys.len() != engine.key_count() {
        return Err(EvolutionError::Config(format!(
            "Initial layout size {} does not match engine key count {}",
            layout.keys.len(),
            engine.key_count()
        )));
    }

    // Apply pinned keys to the initial layout if provided
    if let Some(pinned) = pinned_keys {
        for (i, &p) in pinned.iter().enumerate() {
            if let Some(code) = p {
                if i < layout.keys.len() {
                    if let Some(pos) = layout.keys.iter().position(|&k| k == code) {
                        layout.keys.swap(i, pos);
                    } else {
                        return Err(EvolutionError::Config(format!(
                            "Pinned key {code} not found in initial layout"
                        )));
                    }
                }
            }
        }
    }

    match config {
        keyforge_model::SearchConfig::Annealing {
            steps,
            start_temp,
            end_temp,
            seed,
            patience,
            reheats,
            reheat_factor,
            ..
        } => {
            let mutation = GroupMutation {
                unlocked_indices,
                start_temp: *start_temp,
                end_temp: *end_temp,
            };
            let acceptance = CoolingAnnealing;

            let annealing_config = AnnealingConfig::new(
                *steps,
                *start_temp,
                *end_temp,
                *seed,
                *patience,
                *reheats,
                *reheat_factor,
            )?;

            let mut optimizer = Optimizer::new(
                engine,
                annealing_config,
                mutation,
                acceptance,
                RealTimeKeeper,
            );

            let best_layout = optimizer.run(Some(layout), callback)?;

            Ok(OptimizationResult {
                score: engine.score(&best_layout)?,
                layout: best_layout,
            })
        }
    }
}
