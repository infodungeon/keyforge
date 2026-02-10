#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::{
    types::{FingerIndex, HandIndex, KeyCode},
    Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric,
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
    let dto: keyforge_protocol::CostModelDto = serde_json::from_str(json).unwrap();
    dto.into()
}

fn main() {
    let keys: Vec<KeyNode> = (0u16..2)
        .map(|i| KeyNode {
            index: i.into(),
            hand: HandIndex::new(0),
            finger: FingerIndex::new_unchecked(1), // All index
            x: keyforge_model::types::SpatialUnit::from_f32(i as f32),
            y: keyforge_model::types::SpatialUnit::default(),
            ..Default::default()
        })
        .collect();
    let keyboard = Arc::new(
        Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap(),
    );
    let mut corpus_val = Corpus::default();
    let mut char_freqs = corpus_val.char_freqs.to_vec();
    char_freqs[97] = 1; // 'a'
    char_freqs[98] = 1; // 'b'
    corpus_val.char_freqs = Arc::from(char_freqs);
    corpus_val.bigrams = Arc::from(vec![(97, 98, 1000)]);
    let corpus = Arc::new(corpus_val);

    let layout = Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98)]);
    let cost_model = Arc::new(mock_cost_model());

    // 1. Default Rubric
    let rubric_def = Arc::new(Rubric::default());
    let engine_def = EngineFactory::new_generic(&EngineCompilationContext {
        keyboard: keyboard.clone(),
        corpus: corpus.clone(),
        rubric: rubric_def,
        cost_model: cost_model.clone(),
        engine_config: keyforge_model::config::EngineConfig::default(),
    })
    .unwrap();
    let score_def = engine_def.score(&layout).unwrap();

    // 2. Custom Rubric (High SFB Base)
    let rubric_custom = Arc::new(Rubric::builder().sfb_base(5_000_000_000).build());
    let engine_custom = EngineFactory::new_generic(&EngineCompilationContext {
        keyboard,
        corpus,
        rubric: rubric_custom,
        cost_model,
        engine_config: keyforge_model::config::EngineConfig::default(),
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
