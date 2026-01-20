use keyforge_model::{
    types::{FingerIndex, HandIndex},
    Corpus, CostModel, KeyNode, Keyboard, Rubric,
};
use keyforge_physics::EngineFactory;

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
            finger: FingerIndex(i as u8),
            x: i as f32,
            ..Default::default()
        })
        .collect();
    let kb = Keyboard::new(keys, 0).unwrap();
    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let cost_model = mock_cost_model();

    let _engine = EngineFactory::new_generic(&kb, &corpus, &rubric, &cost_model).unwrap();
    println!("Engine built successfully");
}
