#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::{
    types::{FingerIndex, HandIndex},
    Corpus, CostModel, KeyNode, Keyboard, Rubric,
};
use keyforge_physics::{EngineCompilationContext, EngineFactory};
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

fn main() {
    let keys: Vec<KeyNode> = (0..5)
        .map(|i| KeyNode {
            index: i,
            hand: HandIndex(0),
            finger: FingerIndex::new_unchecked(i as u8),
            x: i as f32,
            ..Default::default()
        })
        .collect();
    let keyboard = Arc::new(Keyboard::new(keys, 0, "test".into()).unwrap());
    let corpus = Arc::new(Corpus::default());
    let rubric = Arc::new(Rubric::default());
    let cost_model = Arc::new(mock_cost_model());

    let engine = EngineFactory::new_generic(&EngineCompilationContext { keyboard,
    corpus,
    rubric,
    cost_model, engine_config: keyforge_model::config::EngineConfig::default() })
    .expect("Failed to build engine");
    println!("Engine built successfully with {} keys", engine.key_count());
}
