// libs/keyforge-evolution/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # KeyForge Evolution
//!
//! The optimization engine for KeyForge. This crate implements meta-heuristic 
//! search algorithms (like Simulated Annealing) to evolve keyboard layouts 
//! toward a minimum score.

pub use errors::EvolutionError;
pub mod supervisor;
pub mod errors;

#[cfg(test)]
mod tests_integration;

use keyforge_model::{Layout, SearchConfig, KeyCode, OptimizationResult};
use keyforge_physics::{EngineRequest, ScoringEngine};
use std::sync::Arc;
use supervisor::AnnealingConfig;
use supervisor::strategies::{CoolingAnnealing, GroupMutation};
use supervisor::traits::RealTimeKeeper;
use supervisor::Optimizer;

/// Trait for receiving progress updates during optimization.
pub trait ProgressCallback: Send + Sync {
    /// Called periodically with the current optimization state.
    /// Returns `true` to continue, `false` to abort.
    fn on_progress(&self, epoch: usize, score: f32, layout: &[KeyCode], ips: f32) -> bool;
}

/// A progress callback that does nothing.
#[derive(Debug)]
pub struct NoOpCallback;
impl ProgressCallback for NoOpCallback {
    fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
        true
    }
}

/// Optimizes a keyboard layout based on an `EngineRequest`.
///
/// This function is a convenience wrapper that creates a `ScoringEngine` internally.
/// For repeated optimizations, consider using `evolve` with a pre-compiled engine.
pub fn optimize(req: &EngineRequest) -> Result<OptimizationResult, EvolutionError> {
    optimize_with_callback(req, NoOpCallback)
}

/// Optimizes a keyboard layout with a progress callback.
pub fn optimize_with_callback<CB: ProgressCallback>(
    req: &EngineRequest,
    callback: CB,
) -> Result<OptimizationResult, EvolutionError> {
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_matrix)?;
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

/// Performs optimization using a pre-compiled `ScoringEngine`.
///
/// This is the recommended entry point for performance-sensitive applications
/// that need to run multiple optimizations against the same parameters.
pub fn evolve<CB: ProgressCallback>(
    engine: Arc<ScoringEngine>,
    config: &SearchConfig,
    callback: CB,
    initial_layout: Option<Layout>,
    pinned_keys: Option<&[Option<KeyCode>]>,
) -> Result<OptimizationResult, EvolutionError> {
    // Determine unlocked indices
    let unlocked_indices: Vec<usize> = (0..engine.key_count())
        .filter(|&i| {
            match pinned_keys {
                Some(pins) => i >= pins.len() || pins[i].is_none(),
                None => true,
            }
        })
        .collect();

    evolve_internal(engine, config, unlocked_indices, initial_layout, callback, pinned_keys)
}

/// Internal helper to share logic between legacy and new entry points.
fn evolve_internal<CB: ProgressCallback>(
    engine: Arc<ScoringEngine>,
    config: &SearchConfig,
    unlocked_indices: Vec<usize>,
    initial_layout: Option<Layout>,
    callback: CB,
    pinned_keys: Option<&[Option<KeyCode>]>,
) -> Result<OptimizationResult, EvolutionError> {
    let mut layout = initial_layout.unwrap_or_else(|| {
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
                    // Swap this key with wherever it currently is to maintain layout integrity
                    // INVARIANT: Permutation integrity must be preserved.
                    if let Some(pos) = layout.keys.iter().position(|&k| k == code) {
                        layout.keys.swap(i, pos);
                    } else {
                        // If the key is missing from the initial layout, we cannot proceed safely
                        // without violating the permutation invariant.
                        return Err(EvolutionError::Config(format!("Pinned key {} not found in initial layout", code)));
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

            // We pass &*engine to dereference the Arc to a reference
            let mut optimizer = Optimizer::new(
                &engine,
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
            initial_layout: None, pinned_keys: vec![], cost_matrix: vec![],
        };
        let result = optimize(&req).unwrap();
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
            cost_matrix: vec![],
        };
        let result = optimize(&req).unwrap();
        assert_eq!(result.layout.keys[0], KeyCode(1));
    }

    #[test]
    fn test_optimize_with_callback_termination() {
        let (kb, cp, rb) = setup_env();
        let req = EngineRequest {
            keyboard: kb, corpus: cp, rubric: rb,
            config: SearchConfig::Annealing { steps: 5000, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 },
            initial_layout: None, pinned_keys: vec![], cost_matrix: vec![],
        };
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        #[derive(Debug)]
        struct CountingCallback { counter: Arc<std::sync::atomic::AtomicUsize>, limit: usize }
        impl ProgressCallback for CountingCallback {
            fn on_progress(&self, _step: usize, _score: f32, _layout: &[KeyCode], _ips: f32) -> bool {
                let val = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                val < self.limit
            }
        }
        let callback = CountingCallback { counter: counter.clone(), limit: 1 };
        let result = optimize_with_callback(&req, callback);
        assert!(matches!(result, Err(EvolutionError::Aborted)));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_evolve_api_direct() {
        let (kb, cp, rb) = setup_env();
        let engine = Arc::new(ScoringEngine::new(&kb, &cp, &rb, &[]).unwrap());
        let config = SearchConfig::Annealing { steps: 10, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 };
        let result = evolve(engine, &config, NoOpCallback, None, None).unwrap();
        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_pinned_key_swap() {
        let (kb, cp, rb) = setup_env();
        let pinned = vec![Some(KeyCode(2)), None, None];
        let req = EngineRequest {
            keyboard: kb, corpus: cp, rubric: rb,
            config: SearchConfig::Annealing { steps: 10, start_temp: 10.0, end_temp: 1.0, seed: 123, patience: 100, reheats: 0, reheat_factor: 1.0 },
            initial_layout: None, pinned_keys: pinned, cost_matrix: vec![],
        };
        let result = optimize(&req).unwrap();
        assert_eq!(result.layout.keys[0], KeyCode(2));
        assert_eq!(result.layout.keys[2], KeyCode(0));
    }

    #[test]
    fn test_error_on_missing_pin() {
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
            initial_layout: None, pinned_keys: pinned, cost_matrix: vec![],
        };
        let result = optimize(&req);
        assert!(result.is_err());
        match result {
            Err(EvolutionError::Config(msg)) => assert!(msg.contains("Pinned key 99 not found")),
            _ => panic!("Expected Config error"),
        }
    }
}
