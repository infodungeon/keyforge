// libs/keyforge-physics/tests/api_integration.rs

use keyforge_model::{
    types::{FingerIndex, HandIndex, KeyCode},
    Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric, SearchConfig, EngineRequest,
};
use std::sync::Arc;

fn setup_kb_wiring() -> Keyboard {
    let keys: Vec<KeyNode> = (0..3)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{i}"),
            hand: HandIndex(0),
            finger: FingerIndex::new_unchecked(i as u8),
            x: i as f32,
            ..Default::default()
        })
        .collect();
    Keyboard::new(keys, 0, "test".into()).unwrap()
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
    let mut corpus = Corpus::default();
    corpus.char_freqs[97] = 100;
    let corpus = Arc::new(corpus);
    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99)]);

    let req = EngineRequest {
        keyboard: kb,
        corpus,
        rubric: Arc::new(Rubric::default()),
        cost_model: Arc::new(mock_cost_model_wiring()),
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
    assert!(result.score > 0.0);

    // Test analyze wrapper
    let report = keyforge_compute::analyze(&req).unwrap();
    assert!(report.score > 0.0);

    // Test identify
    let identity = keyforge_physics::identify(&req.initial_layout.clone().unwrap());
    let _ = identity; // Might be None if not matched, but we call it
}
