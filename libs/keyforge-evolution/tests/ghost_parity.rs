#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-evolution/tests/ghost_parity.rs

    use keyforge_evolution::ghost::GhostOptimizer;
    use keyforge_evolution::{evolve, NoOpCallback};
    use keyforge_model::{
        types::{FingerIndex, HandIndex, KeyCode, RowIndex},
        Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric, SearchConfig,
    };
    use keyforge_physics::{EngineCompilationContext, EngineFactory, ScoringEngine};
    use std::sync::Arc;

    fn mock_cost_model() -> CostModel {
        let json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0 },
                        "index": { "base": { "r0": 100.0 } },
                        "middle": { "base": { "r0": 100.0 } },
                        "ring": { "base": { "r0": 100.0 } },
                        "pinky": { "base": { "r0": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
        serde_json::from_str(json).unwrap()
    }

    fn setup_env() -> (Arc<dyn ScoringEngine>, SearchConfig, Layout) {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                row: RowIndex(0),
                x: 0.0,
                y: 0.0,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex(2),
                row: RowIndex(0),
                x: 1.0, // Distance 1.0
                y: 0.0,
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let mut corpus = Corpus::default();
        corpus.char_freqs[0] = 100;
        corpus.char_freqs[1] = 100;
        corpus.bigrams.push((0, 1, 100)); // Key 0 -> Key 1

        let mut rubric = Rubric::default();
        rubric.travel_lat = 1.0;

        let cm = mock_cost_model();

        let engine = EngineFactory::new_scalar(EngineCompilationContext {
            keyboard: &kb,
            corpus: &corpus,
            rubric: &rubric,
            cost_model: &cm,
        })
        .unwrap();

        let config = SearchConfig::Annealing {
            steps: 100,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 10,
            reheats: 0,
            reheat_factor: 0.5,
            include_thumbs: false,
        };

        let layout = Layout::new_unchecked(vec![KeyCode(0), KeyCode(1)]);
        (Arc::from(engine), config, layout)
    }

    #[test]
    fn test_ghost_parity_deterministic() {
        let (engine, config, layout) = setup_env();

        // Run Production Optimizer
        let prod_res = evolve(&engine, &config, NoOpCallback, Some(layout.clone()), None).unwrap();

        // Run Ghost Optimizer
        let ghost_res = GhostOptimizer::optimize(engine.as_ref(), &config, &layout).unwrap();

        assert!(
            prod_res.score > 0.0,
            "Production score should be > 0 (Actual: {})",
            prod_res.score
        );
        assert!(
            ghost_res.score > 0.0,
            "Ghost score should be > 0 (Actual: {})",
            ghost_res.score
        );

        // Ghost and Prod should produce similar results (not necessarily identical due to RNG)
        // But for a 2-key layout, they should likely both stay at optimal or swap.
        // Optimal is likely current layout (distance 1.0) vs swapped (distance 1.0).
        // Actually if keys are 0 and 1, bigram 0->1.
        // If layout is [0, 1]: Key 0 at x=0, Key 1 at x=1. Dist=1.
        // If layout is [1, 0]: Key 1 at x=0, Key 0 at x=1. Dist=1.
        // Score should be same?
        // Static costs are both 100.0.
        // So both layouts are equivalent.

        let score_diff = (prod_res.score - ghost_res.score).abs();
        // Allow some variance if they converged to different local minima (not possible here, but generally)
        assert!(score_diff < 1.0, "Scores should be close");
    }
}
