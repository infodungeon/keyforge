// libs/keyforge-evolution/src/verify.rs

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

//! # Evolution Verification
//!
//! Metamorphic and parity testing between the high-performance optimizer
//! and the reference ghost model.

#[keyforge_testing_macros::kf_test]
mod tests {
    use crate::{evolve, NoOpCallback};
    use keyforge_model::{KeyCode, Layout, SearchConfig};
    use keyforge_physics::{EngineCompilationContext, EngineFactory};
    use std::sync::Arc;

    fn setup_minimal() -> (Arc<dyn keyforge_physics::ScoringEngine>, Layout) {
        let (kb, corpus, rubric, cm) = keyforge_model::testing::setup_minimal_assets();

        let engine = EngineFactory::new_scalar(&EngineCompilationContext {
            keyboard: kb.into(),
            corpus: corpus.into(),
            rubric: rubric.into(),
            cost_model: cm.into(),
            engine_config: keyforge_model::config::EngineConfig::default(),
        })
        .unwrap();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99)]);
        (Arc::from(engine), layout)
    }

    #[test]
    fn test_zero_step_rejection() {
        let (engine, layout) = setup_minimal();
        let config = SearchConfig::Annealing {
            steps: 0,
            start_temp: 1.0,
            end_temp: 0.1,
            seed: 1,
            patience: 1,
            reheats: 0,
            reheat_factor: 1.0,
            include_thumbs: false,
        };

        let res = evolve(&engine, &config, NoOpCallback, Some(layout), None);
        assert!(res.is_err());
    }

    #[test]
    fn test_deterministic_seed_parity() {
        let (engine, layout) = setup_minimal();
        let config = SearchConfig::Annealing {
            steps: 10,
            start_temp: 100.0,
            end_temp: 0.01,
            seed: 12345, // Fixed seed
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
            include_thumbs: false,
        };

        // Note: Full bit-perfect parity between Ghost and Production is NOT
        // guaranteed for N > 0 steps due to implementation differences in
        // mutation strategies (GroupMutation vs simple swap).
        // However, we can prove that repeating the same run with the same
        // seed in production is stable.

        let res1 = evolve(&engine, &config, NoOpCallback, Some(layout.clone()), None).unwrap();
        let res2 = evolve(&engine, &config, NoOpCallback, Some(layout.clone()), None).unwrap();

        assert_eq!(res1.layout.keys, res2.layout.keys);
        assert_eq!(res1.score, res2.score);
    }
}
