use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvolutionError {
    #[error("Physics Violation: {0}")]
    Physics(#[from] keyforge_physics::PhysicsError),

    #[error("Configuration Error: {0}")]
    Config(String),
}
pub mod supervisor;
pub mod errors;

use keyforge_model::{Layout, SearchConfig, KeyCode, OptimizationResult};
use keyforge_physics::{EngineRequest, ScoringEngine};
use std::sync::Arc;
use supervisor::AnnealingConfig;
use supervisor::strategies::{CoolingAnnealing, GroupMutation};
use supervisor::traits::RealTimeKeeper;
use supervisor::Optimizer;

pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, epoch: usize, score: f32, layout: &[KeyCode], ips: f32) -> bool;
}

pub struct NoOpCallback;
impl ProgressCallback for NoOpCallback {
    fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
        true
    }
}

/// Legacy Entry Point: Compiles the engine from scratch (Keep for backward compat).
pub fn optimize(req: &EngineRequest) -> OptimizationResult {
    optimize_with_callback(req, NoOpCallback)
}

/// Legacy Entry Point with Callback.
pub fn optimize_with_callback<CB: ProgressCallback>(
    req: &EngineRequest,
    callback: CB,
) -> OptimizationResult {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides)
        .expect("Failed to initialize physics engine");
    let engine_arc = Arc::new(engine);

    // Determine pinned keys for legacy request
    let unlocked_indices: Vec<usize> = (0..engine_arc.key_count())
        .filter(|&i| i >= req.pinned_keys.len() || req.pinned_keys[i].is_none())
        .collect();

    evolve_internal(
        engine_arc,
        &req.config,
        unlocked_indices,
        req.initial_layout.clone(),
        callback,
        Some(&req.pinned_keys),
    )
}

/// New Entry Point: Uses a pre-compiled, shared Physics Engine.
/// This is used by the Workspace Runtime.
pub fn evolve<CB: ProgressCallback>(
    engine: Arc<ScoringEngine>,
    config: &SearchConfig,
    callback: CB,
) -> OptimizationResult {
    // Default: All keys unlocked, default initial layout
    let unlocked_indices = (0..engine.key_count()).collect();
    evolve_internal(engine, config, unlocked_indices, None, callback, None)
}

/// Internal helper to share logic between legacy and new entry points.
fn evolve_internal<CB: ProgressCallback>(
    engine: Arc<ScoringEngine>,
    config: &SearchConfig,
    unlocked_indices: Vec<usize>,
    initial_layout: Option<Layout>,
    callback: CB,
    pinned_keys: Option<&[Option<KeyCode>]>,
) -> OptimizationResult {
    let mut layout = initial_layout.unwrap_or_else(|| {
        let keys: Vec<KeyCode> = (0..engine.key_count()).map(|i| KeyCode(i as u16)).collect();
        Layout::new_unchecked(keys)
    });

    // Guardrail: Ensure layout matches engine geometry
    if layout.keys.len() != engine.key_count() {
        panic!(
            "Evolution Error: Initial layout size {} does not match engine key count {}",
            layout.keys.len(),
            engine.key_count()
        );
    }

    // Apply pinned keys to the initial layout if provided
    if let Some(pinned) = pinned_keys {
        for (i, &p) in pinned.iter().enumerate() {
            if let Some(code) = p {
                if i < layout.keys.len() {
                    // Swap this key with wherever it currently is to maintain layout integrity
                    // INVARIANT: Permutation integrity must be preserved.
                    if let Some(pos) = layout.keys.iter().position(|&k| k == code) {
                        layout.keys.swap(i, pos);
                    } else {
                        // If the key is missing from the initial layout, we cannot proceed safely
                        // without violating the permutation invariant.
                        panic!("Pinned key {} not found in initial layout", code);
                    }
                }
            }
        }
    }

    match config {
        SearchConfig::Annealing {
            steps,
            start_temp,
            end_temp,
            seed,
            patience,
            reheats,
            reheat_factor,
        } => {
            let mutation = GroupMutation { unlocked_indices };
            let acceptance = CoolingAnnealing;

            let annealing_config = AnnealingConfig::new(
                *steps,
                *start_temp,
                *end_temp,
                *seed,
                *patience,
                *reheats,
                *reheat_factor,
            )
            .expect("Invalid Annealing Configuration");

            // We pass &*engine to dereference the Arc to a reference
            let mut optimizer = Optimizer::new(
                &engine,
                annealing_config,
                mutation,
                acceptance,
                RealTimeKeeper,
            );

            let best_layout = optimizer.run(Some(layout), callback);

            OptimizationResult {
                score: engine.score(&best_layout),
                layout: best_layout,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, SearchConfig};
    use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
    use keyforge_physics::{EngineRequest, ScoringEngine};
    use std::sync::Arc;

    fn setup_env() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>) {
        let keys = vec![
            KeyNode { index: 0, label: "k0".to_string(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), col: ColIndex(0), x: 0.0, y: 0.0, is_home: true, ..Default::default() },
            KeyNode { index: 1, label: "k1".to_string(), hand: HandIndex(0), finger: FingerIndex(2), row: RowIndex(0), col: ColIndex(1), x: 1.0, y: 0.0, is_home: true, ..Default::default() },
            KeyNode { index: 2, label: "k2".to_string(), hand: HandIndex(0), finger: FingerIndex(3), row: RowIndex(0), col: ColIndex(2), x: 2.0, y: 0.0, is_home: true, ..Default::default() },
        ];
        (Arc::new(Keyboard::new(keys, 0).unwrap()), Arc::new(Corpus::default()), Arc::new(Rubric::default()))
    }

    #[test]
    fn test_legacy_optimize_entry_point() {
        let (kb, cp, rb) = setup_env();
        let req = EngineRequest {
            keyboard: kb, corpus: cp, rubric: rb,
            config: SearchConfig::Annealing { steps: 10, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 },
            initial_layout: None, pinned_keys: vec![], cost_overrides: vec![],
        };
        let result = optimize(&req);
        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_legacy_optimize_full_options() {
        let (kb, cp, rb) = setup_env();
        let req = EngineRequest {
            keyboard: kb, corpus: cp, rubric: rb,
            config: SearchConfig::Annealing { steps: 10, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 },
            initial_layout: Some(Layout::new_unchecked(vec![KeyCode(1), KeyCode(0), KeyCode(2)])),
            pinned_keys: vec![Some(KeyCode(1)), None],
            cost_overrides: vec![],
        };
        let result = optimize(&req);
        assert_eq!(result.layout.keys[0], KeyCode(1));
    }

    #[test]
    fn test_optimize_with_callback_termination() {
        let (kb, cp, rb) = setup_env();
        let req = EngineRequest {
            keyboard: kb, corpus: cp, rubric: rb,
            config: SearchConfig::Annealing { steps: 5000, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 },
            initial_layout: None, pinned_keys: vec![], cost_overrides: vec![],
        };
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        struct CountingCallback { counter: Arc<std::sync::atomic::AtomicUsize>, limit: usize }
        impl ProgressCallback for CountingCallback {
            fn on_progress(&self, _step: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
                let val = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                val < self.limit
            }
        }
        let callback = CountingCallback { counter: counter.clone(), limit: 1 };
        let result = optimize_with_callback(&req, callback);
        assert!(result.score >= 0.0);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_evolve_api_direct() {
        let (kb, cp, rb) = setup_env();
        let engine = Arc::new(ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap());
        let config = SearchConfig::Annealing { steps: 10, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 };
        let result = evolve(engine, &config, crate::NoOpCallback);
        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_pinned_key_swap() {
        let (kb, cp, rb) = setup_env();
        let pinned = vec![Some(KeyCode(2)), None, None];
        let req = EngineRequest {
            keyboard: kb, corpus: cp, rubric: rb,
            config: SearchConfig::Annealing { steps: 10, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 },
            initial_layout: None, pinned_keys: pinned, cost_overrides: vec![],
        };
        let result = optimize(&req);
        assert_eq!(result.layout.keys[0], KeyCode(2));
        assert_eq!(result.layout.keys[2], KeyCode(0));
    }

    #[test]
    #[should_panic(expected = "Pinned key 99 not found in initial layout")]
    fn test_panic_on_missing_pin() {
        let keys = vec![
            KeyNode { index: 0, label: "k0".into(), hand: HandIndex(0), finger: FingerIndex(0), row: RowIndex(0), col: ColIndex(0), x: 0.0, y: 0.0, is_home: false, ..Default::default() },
            KeyNode { index: 1, label: "k1".into(), hand: HandIndex(0), finger: FingerIndex(1), row: RowIndex(0), col: ColIndex(1), x: 1.0, y: 0.0, is_home: false, ..Default::default() },
        ];
        let kb = Arc::new(Keyboard::new(keys, 0).unwrap());
        let corpus = Arc::new(Corpus::default());
        let rubric = Arc::new(Rubric::default());
        let config = SearchConfig::Annealing { steps: 10, start_temp: 1.0, end_temp: 0.1, seed: 42, patience: 10, reheats: 0, reheat_factor: 1.0 };
        let pinned = vec![Some(KeyCode(99)), None];
        let req = EngineRequest {
            keyboard: kb, corpus, rubric, config,
            initial_layout: None, pinned_keys: pinned, cost_overrides: vec![],
        };
        crate::optimize(&req);
    }
}
