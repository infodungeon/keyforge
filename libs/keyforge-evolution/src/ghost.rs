// libs/keyforge-evolution/src/ghost.rs

use crate::errors::EvolutionError;
use crate::ProgressCallback;
use keyforge_model::types::KeyIndex;
use keyforge_model::{Layout, Score};
use keyforge_physics::ScoringEngine;
use rand::Rng;
use std::time::Instant;

/// A simple, reference implementation of a local search optimizer (Hill Climbing).
/// Intended for mathematical verification and baseline performance comparison.
#[derive(Debug, Clone, Copy)]
pub struct GhostHillClimber;

impl GhostHillClimber {
    /// Runs a hill-climbing search from the initial layout.
    ///
    /// # Errors
    /// Returns `EvolutionError` if scoring fails.
    #[allow(clippy::cast_precision_loss)]
    pub fn run<CB: ProgressCallback>(
        &self,
        engine: &dyn ScoringEngine,
        initial_layout: Layout,
        steps: usize,
        callback: &CB,
    ) -> Result<Layout, EvolutionError> {
        let mut current_layout = initial_layout;
        let mut current_score = engine.score(&current_layout)?;
        let mut best_layout = current_layout.clone();
        let mut best_score = current_score;

        let start_time = Instant::now();
        let mut rng = rand::thread_rng();

        for step in 0..steps {
            // 1. Propose a random swap
            let key_count = engine.key_count();
            if key_count < 2 {
                break;
            }

            let a = rng.gen_range(0..key_count);
            let mut b = rng.gen_range(0..key_count);
            while a == b {
                b = rng.gen_range(0..key_count);
            }

            let mut next_layout = current_layout.clone();
            next_layout
                .swap(
                    KeyIndex::new(u16::try_from(a).map_err(|_| {
                        EvolutionError::Internal("Key index overflow".into())
                    })?),
                    KeyIndex::new(u16::try_from(b).map_err(|_| {
                        EvolutionError::Internal("Key index overflow".into())
                    })?),
                )
                .map_err(|e| EvolutionError::Internal(e.to_string()))?;

            let next_score = engine.score(&next_layout)?;

            // 2. Acceptance (Strictly improving)
            if next_score < current_score {
                current_layout = next_layout;
                current_score = next_score;

                if current_score < best_score {
                    best_layout = current_layout.clone();
                    best_score = current_score;
                }
            }

            // 3. Callback (Periodically)
            if step % 1000 == 0 {
                let elapsed = start_time.elapsed().as_secs_f32();
                let ips = if elapsed > 0.0 {
                    step as f32 / elapsed
                } else {
                    0.0
                };

                let score_f32 = best_score.raw() as f32 / 1_000_000.0;
                let keys = best_layout.keys();

                if callback.on_progress(step, score_f32, keys, ips)
                    != crate::OptimizationControl::Continue
                {
                    return Err(EvolutionError::Aborted);
                }
            }
        }

        Ok(best_layout)
    }
}

/// Baseline result for verification tests.
#[derive(Debug, Clone)]
pub struct GhostResult {
    /// Final best layout found.
    pub layout: Layout,
    /// Final best score achieved.
    pub score: Score,
}