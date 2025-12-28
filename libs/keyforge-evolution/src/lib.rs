pub mod supervisor;

use keyforge_model::OptimizationResult;
use keyforge_model::{Layout, SearchConfig};
use keyforge_physics::{EngineRequest, ScoringEngine};
use std::sync::Arc;
use supervisor::strategies::{CoolingAnnealing, GroupMutation};
use supervisor::Optimizer;

pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, epoch: usize, score: f32, layout: &[u16], ips: f32) -> bool;
}

pub struct NoOpCallback;
impl ProgressCallback for NoOpCallback {
    fn on_progress(&self, _epoch: usize, _score: f32, _layout: &[u16], _ips: f32) -> bool {
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
    let engine = ScoringEngine::new(&req.keyboard, &req.corpus, &req.rubric, &req.cost_overrides);
    let engine_arc = Arc::new(engine);

    // Determine pinned keys for legacy request
    let unlocked_indices: Vec<usize> = (0..engine_arc.key_count())
        .filter(|&i| i >= req.pinned_keys.len() || req.pinned_keys[i].is_none())
        .collect();

    evolve_internal(engine_arc, &req.config, unlocked_indices, req.initial_layout.clone(), callback, Some(&req.pinned_keys))
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
    pinned_keys: Option<&[Option<u16>]>,
) -> OptimizationResult {
    let mut layout = initial_layout.unwrap_or_else(|| {
        let keys: Vec<u16> = (0..engine.key_count()).map(|i| i as u16).collect();
        Layout::new(keys)
    });

    // Apply pinned keys to the initial layout if provided
    if let Some(pinned) = pinned_keys {
        for (i, &p) in pinned.iter().enumerate() {
            if let Some(code) = p {
                if i < layout.keys.len() {
                    // Swap this key with wherever it currently is to maintain layout integrity
                    // (though evolve_internal doesn't strictly require integrity, it's good practice)
                    if let Some(pos) = layout.keys.iter().position(|&k| k == code) {
                        layout.keys.swap(i, pos);
                    } else {
                        layout.keys[i] = code;
                    }
                }
            }
        }
    }

    match config {
        SearchConfig::Annealing { steps, start_temp, end_temp, seed, patience, reheats, reheat_factor } => {
            let mutation = GroupMutation { unlocked_indices };
            let acceptance = CoolingAnnealing;

            // We pass &*engine to dereference the Arc to a reference
            let mut optimizer = Optimizer::new(
                &engine,
                *steps,
                *start_temp,
                *end_temp,
                *seed,
                mutation,
                acceptance,
                *patience,
                *reheats,
                *reheat_factor,
            );

            let best_layout = optimizer.run(Some(layout), callback);

            OptimizationResult {
                score: engine.score(&best_layout),
                layout: best_layout,
            }
        }
    }
}
