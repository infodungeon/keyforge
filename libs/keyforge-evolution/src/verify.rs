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

#[cfg(test)]
mod tests {
    use crate::ghost::GhostOptimizer;
    use crate::{evolve, NoOpCallback};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric, SearchConfig, KeyCode};
    use keyforge_physics::EngineFactory;
    use std::sync::Arc;
    use std::collections::HashMap;

    fn setup_minimal() -> (Arc<dyn keyforge_physics::ScoringEngine>, Layout) {
        let keys = vec![
            KeyNode { index: 0, ..Default::default() },
            KeyNode { index: 1, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        
        let mut cm = CostModel::default();
        let mut fingers = HashMap::new();
        fingers.insert(
            "index".to_string(),
            keyforge_model::cost_model::FingerDefinition::Standard(HashMap::from([
                ("base".to_string(), HashMap::from([("r0".to_string(), 1.0)])),
            ])),
        );
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );

        let engine = EngineFactory::new_scalar(
            &kb, 
            &Corpus::default(), 
            &Rubric::default(), 
            &cm
        ).unwrap();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);
        (Arc::from(engine), layout)
    }

    #[test]
    fn test_zero_step_invariance() {
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

        // Production
        let res_prod = evolve(&engine, &config, NoOpCallback, Some(layout.clone()), None).unwrap();
        
        // Ghost
        let res_ghost = GhostOptimizer::optimize(engine.as_ref(), &config, &layout);

        // Invariant: At 0 steps, neither should change the layout
        assert_eq!(res_prod.layout.keys, layout.keys);
        assert_eq!(res_ghost.layout.keys, layout.keys);
        assert_eq!(res_prod.score, res_ghost.score);
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