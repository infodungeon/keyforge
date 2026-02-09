#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-physics/tests/api_integration.rs
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode};
    use keyforge_model::{
        Corpus, CostModel, EngineRequest, KeyNode, Keyboard, Layout, Rubric, SearchConfig,
    };
    use std::sync::Arc;

    fn setup_kb_wiring() -> Keyboard {
        let keys: Vec<KeyNode> = (0..3)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{i}"),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(i as u8),
                x: keyforge_model::types::SpatialUnit::from_f32(i as f32),
                ..Default::default()
            })
            .collect();
        Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap()
    }

    fn mock_cost_model_wiring() -> CostModel {
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

    #[test]
    fn test_public_api_wrappers() {
        let kb = Arc::new(setup_kb_wiring());
        let mut corpus_val = Corpus::default();
        let mut char_freqs = corpus_val.char_freqs.to_vec();
        char_freqs[97] = 100;
        corpus_val.char_freqs = Arc::from(char_freqs);
        let corpus = Arc::new(corpus_val);
        let layout =
            Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98), KeyCode::new(99)]);

        let req = EngineRequest {
            keyboard: kb,
            corpus,
            rubric: Arc::new(Rubric::default()),
            cost_model: Arc::new(mock_cost_model_wiring()),
            engine_config: keyforge_model::config::EngineConfig::default(),
            config: SearchConfig::Annealing {
                steps: 100,
                start_temp: 10.0,
                end_temp: 0.1,
                seed: 42,
                patience: 10,
                reheats: 0,
                reheat_factor: 0.5,
                include_thumbs: false,
            },
            initial_layout: Some(layout),
            pinned_keys: vec![],
        };

        let suggestions = keyforge_compute::suggest_improvements(&req).unwrap();
        assert!(suggestions.len() <= 5);

        // Test score wrapper
        let result = keyforge_compute::score(&req).unwrap();
        assert!(result.score > keyforge_model::types::Score::ZERO);

        // Test analyze wrapper
        let report = keyforge_compute::analyze(&req).unwrap();
        assert!(report.score > keyforge_model::types::Score::ZERO);

        // Test identify
        if let Some(initial) = &req.initial_layout {
            let _ = keyforge_physics::identify(initial);
        }
    }
}
