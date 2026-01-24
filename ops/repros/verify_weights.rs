#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::{
    types::{FingerIndex, HandIndex, KeyCode},
    Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric,
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
    let keys: Vec<KeyNode> = (0..2)
        .map(|i| KeyNode {
            index: i,
            hand: HandIndex(0),
            finger: FingerIndex(1), // All index
            x: i as f32,
            y: 0.0,
            ..Default::default()
        })
        .collect();
    let keyboard = Keyboard::new(keys, 0, "test".into()).unwrap();
    let mut corpus = Corpus::default();
    corpus.char_freqs[97] = 1; // 'a'
    corpus.char_freqs[98] = 1; // 'b'
    corpus.bigrams.push((97, 98, 1000)); // 'a' and 'b'

    let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);
    let cost_model = mock_cost_model();

    // 1. Default Rubric
    let rubric_def = Rubric::default();
    let engine_def = EngineFactory::new_generic(keyforge_physics::EngineCompilationContext {
        keyboard: &keyboard,
        corpus: &corpus,
        rubric: &rubric_def,
        cost_model: &cost_model,
    })
    .unwrap();
    let score_def = engine_def.score(&layout).unwrap();

    // 2. Custom Rubric (High SFB Base)
    let rubric_custom = Rubric {
        sfb_base: 5000.0,
        ..Rubric::default()
    };
    let engine_custom = EngineFactory::new_generic(keyforge_physics::EngineCompilationContext {
        keyboard: &keyboard,
        corpus: &corpus,
        rubric: &rubric_custom,
        cost_model: &cost_model,
    })
    .unwrap();
    let score_custom = engine_custom.score(&layout).unwrap();

    println!("Score Default: {:.4}", score_def.to_f32());
    println!("Score Custom (High SFB): {:.4}", score_custom.to_f32());

    if score_custom > score_def {
        println!("SUCCESS: Custom weights are honored!");
    } else {
        println!("FAILURE: Custom weights had no effect.");
        std::process::exit(1);
    }
}
