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
    let engine = keyforge_physics::EngineFactory::new_generic(&req.keyboard, &req.corpus, &req.rubric, &req.cost_model)?;
    let engine_arc: Arc<dyn ScoringEngine> = engine.into();

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
    engine: &Arc<dyn ScoringEngine>,
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
    engine: &Arc<dyn ScoringEngine>,
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
                engine.as_ref(),
                annealing_config,
                mutation,
                acceptance,
                RealTimeKeeper,
            );

            let best_layout = optimizer.run(Some(layout), callback)?;

            // Re-validate using Exact engine for bit-perfect final report
            let exact_score = engine.score(&best_layout)?;

            Ok(OptimizationResult {
                score: exact_score.to_f32(),
                layout: best_layout,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{KeyNode, Keyboard, Corpus, Rubric, CostModel};
    use keyforge_physics::EngineFactory;

    fn setup_env() -> (Arc<dyn ScoringEngine>, SearchConfig) {
        let kb = Keyboard::new(vec![
            KeyNode { index: 0, ..Default::default() },
            KeyNode { index: 1, ..Default::default() },
        ], 0, "test".into()).unwrap();
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::new(),
        });
        let engine = EngineFactory::new_exact(&kb, &Corpus::default(), &Rubric::default(), &cm).unwrap();
        (Arc::from(engine), SearchConfig::default())
    }

    #[test]
    fn test_evolve_basic() {
        let (engine, config) = setup_env();
        let res = evolve(&engine, &config, NoOpCallback, None, None).unwrap();
        assert_eq!(res.layout.len(), 2);
    }

    #[test]
    fn test_evolve_error_branches() {
        let (engine, config) = setup_env();
        
        // 1. Size mismatch
        let bad_layout = Layout::new_unchecked(vec![KeyCode(0)]);
        let res = evolve(&engine, &config, NoOpCallback, Some(bad_layout), None);
        assert!(res.is_err());

        // 2. Missing pinned key
        let pins = vec![Some(KeyCode(999))];
        let res = evolve(&engine, &config, NoOpCallback, None, Some(&pins));
        assert!(res.is_err());
    }

    #[test]
    fn test_optimize_wrapper() {
        let kb = Keyboard::new(vec![KeyNode::default()], 0, "test".into()).unwrap();
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::new(),
        });
        
        let req = EngineRequest {
            keyboard: Arc::new(kb),
            corpus: Arc::new(Corpus::default()),
            rubric: Arc::new(Rubric::default()),
            cost_model: Arc::new(cm),
            config: SearchConfig::default(),
            initial_layout: None,
            pinned_keys: vec![],
        };
        
        let res = optimize(&req).unwrap();
        assert!(res.score >= 0.0);
    }

    #[test]
    fn test_evolve_with_pinned_keys() {
        let (engine, config) = setup_env();
        // Key 0 is pinned to KeyCode 1.
        // Initial layout is usually [0, 1].
        // Pinning [Some(1)] should result in [1, 0].
        let pins = vec![Some(KeyCode(1))];
        let res = evolve(&engine, &config, NoOpCallback, None, Some(&pins)).unwrap();
        assert_eq!(res.layout.keys[0], KeyCode(1));
    }

    #[test]
    fn test_evolve_with_callback() {
        let (engine, _config) = setup_env();
        let config = SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.5,
            include_thumbs: false,
        };

        use std::sync::atomic::{AtomicUsize, Ordering};
        struct MockCallback(Arc<AtomicUsize>);
        impl ProgressCallback for MockCallback {
            fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> crate::OptimizationControl {
                self.0.fetch_add(1, Ordering::SeqCst);
                crate::OptimizationControl::Continue
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let _ = evolve(&engine, &config, MockCallback(count.clone()), None, None).unwrap();
        
        assert!(count.load(Ordering::SeqCst) > 0);
    }
}
