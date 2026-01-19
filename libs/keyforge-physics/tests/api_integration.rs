// libs/keyforge-physics/tests/api_integration.rs

use keyforge_model::{
    types::{FingerIndex, HandIndex, KeyCode},
    Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric, SearchConfig,
};
use keyforge_physics::{suggest_improvements, EngineRequest};
use std::sync::Arc;

fn setup_kb_wiring() -> Keyboard {
    let keys: Vec<KeyNode> = (0..3)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{}", i),
            hand: HandIndex(0),
            finger: FingerIndex(i as u8),
            x: i as f32,
            ..Default::default()
        })
        .collect();
    Keyboard::new(keys, 0).unwrap()
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
        config: SearchConfig::default(),
        initial_layout: Some(layout),
        pinned_keys: vec![],
    };

    let suggestions = suggest_improvements(&req).unwrap();
    assert!(suggestions.len() <= 5);
}
