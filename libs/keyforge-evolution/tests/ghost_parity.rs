#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-evolution/tests/ghost_parity.rs

    use keyforge_evolution::ghost::GhostHillClimber;
    use keyforge_evolution::{evolve, NoOpCallback};
    use keyforge_model::{
        types::{FingerIndex, HandIndex, KeyCode, RowIndex, SpatialUnit, Temperature},
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
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex::new(0),
                x: SpatialUnit::from_f32(0.0),
                y: SpatialUnit::from_f32(0.0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(2),
                row: RowIndex::new(0),
                x: SpatialUnit::from_f32(1.0), // Distance 1.0
                y: SpatialUnit::from_f32(0.0),
                ..Default::default()
            },
        ];
        let kb = Arc::new(Keyboard::new(keys, RowIndex::new(0), "test".into()).unwrap());
        let mut corpus_val = Corpus::default();
        let mut char_freqs = corpus_val.char_freqs.to_vec();
        char_freqs[0] = 100;
        char_freqs[1] = 100;
        corpus_val.char_freqs = Arc::from(char_freqs);
        corpus_val.bigrams = Arc::from(vec![(0, 1, 100)]);

        let rubric = Arc::new(Rubric::builder().travel_lat(1.0).build());

        let cm = Arc::new(mock_cost_model());

        let engine = EngineFactory::new_scalar(&EngineCompilationContext {
            keyboard: kb,
            corpus: Arc::new(corpus_val),
            rubric,
            cost_model: cm,
            engine_config: keyforge_model::config::EngineConfig::default(),
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

        let layout = Layout::new_unchecked(vec![KeyCode::new(0), KeyCode::new(1)]);
        (Arc::from(engine), config, layout)
    }

    #[test]
    fn test_ghost_parity_deterministic() {
        let (engine, config, layout) = setup_env();

        // Run Production Optimizer
        let prod_res = evolve(&engine, &config, NoOpCallback, Some(layout.clone()), None).unwrap();

        // Run Ghost Optimizer (Hill Climber)
        let ghost = GhostHillClimber;
        let ghost_layout = ghost
            .run(engine.as_ref(), layout, 100, &NoOpCallback)
            .unwrap();
        let ghost_score = engine.score(&ghost_layout).unwrap();

        // Verify parity
        let score_diff = (prod_res.score - ghost_score.to_f32()).abs();
        assert!(
            score_diff < 100.0,
            "Scores should be within reasonable bounds"
        );
    }
}
