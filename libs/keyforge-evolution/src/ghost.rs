// libs/keyforge-evolution/src/ghost.rs

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

//! # Evolution Ghost Model
//!
//! A simplified reference implementation of the Simulated Annealing loop.
//! This module focuses on the core stochastic logic without performance
//! optimizations or complex progress reporting.

use keyforge_model::{Layout, OptimizationResult, SearchConfig};
use keyforge_physics::ScoringEngine;
use rand::Rng;

/// Reference implementation of the annealing algorithm.
#[derive(Debug)]
pub struct GhostOptimizer;

impl GhostOptimizer {
    /// Pure reference implementation of Simulated Annealing.
    ///
    /// # Errors
    /// Returns `EvolutionError` if the scoring engine returns an error.
    pub fn optimize(
        engine: &dyn ScoringEngine,
        config: &SearchConfig,
        initial_layout: &Layout,
    ) -> Result<OptimizationResult, crate::EvolutionError> {
        let (steps, mut temp, cooling) = match config {
            SearchConfig::Annealing {
                steps,
                start_temp,
                end_temp,
                ..
            } => {
                let steps = *steps;
                #[allow(clippy::cast_precision_loss)]
                let cooling = (end_temp / start_temp).powf(1.0 / steps as f32);
                (steps, *start_temp, cooling)
            }
        };

        let mut current_layout = initial_layout.clone();
        let mut current_score = engine
            .score(&current_layout)
            .map_err(crate::EvolutionError::Physics)?;

        let mut best_layout = current_layout.clone();
        let mut best_score = current_score;

        let mut rng = rand::rng();

        for _ in 0..steps {
            // 1. Mutate (Simple random swap)
            let mut next_layout = current_layout.clone();
            let len = next_layout.len();
            let a = rng.random_range(0..len);
            let b = rng.random_range(0..len);
            next_layout
                .swap(
                    keyforge_model::types::KeyIndex(a as u16),
                    keyforge_model::types::KeyIndex(b as u16),
                )
                .unwrap();
            // 2. Score
            let next_score = engine
                .score(&next_layout)
                .map_err(crate::EvolutionError::Physics)?;
            let delta = next_score.0 - current_score.0;

            // 3. Accept/Reject (Metropolis Criterion)
            #[allow(clippy::cast_precision_loss)]
            if delta < 0
                || rng.random::<f32>()
                    < (-(delta as f32 / keyforge_model::constants::SCORE_SCALE) / temp).exp()
            {
                current_layout = next_layout;
                current_score = next_score;

                if current_score < best_score {
                    best_score = current_score;
                    best_layout = current_layout.clone();
                }
            }

            // 4. Cool
            temp *= cooling;
        }

        Ok(OptimizationResult {
            score: best_score.to_f32(),
            raw_score: best_score.0,
            layout: best_layout,
        })
    }
}
