use keyforge_compute::SessionBuilder;
use keyforge_infra::{AssetLoader, FsProvider};
use keyforge_model::{types::KeyCode, Corpus, CostModel, KeyboardDefinition, Layout};
use keyforge_protocol::BiometricSample;
use std::path::PathBuf;
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

#[tokio::main]
async fn main() {
    let data_dir = PathBuf::from("data");
    let loader = FsProvider::new(data_dir);

    let kb_name = "ortho_30";
    let kb_def = loader.load::<KeyboardDefinition>(kb_name).await.unwrap();
    let cost_model = Arc::new(mock_cost_model());

    // Create a manual corpus with 1 bigram 'th' (116, 104)
    let mut corpus = Corpus::default();
    corpus.char_freqs[116] = 1;
    corpus.char_freqs[104] = 1;
    corpus.bigrams.push((116, 104, 1000));
    let corpus_arc = Arc::new(corpus);

    // 1. Without Biometrics
    let builder_none = SessionBuilder::new(&loader)
        .with_keyboard_def(kb_def.clone())
        .with_cost_model_obj(cost_model.clone())
        .with_corpus_obj(corpus_arc.clone());
    let session_none = builder_none.build().unwrap();
    let engine_none = session_none.engine;

    // 2. With "High Latency" Biometrics for 'th' bigram
    let biometrics = vec![
        BiometricSample {
            bigram: "th".to_string(),
            ms: 500.0,
            timestamp: 0,
        },
        BiometricSample {
            bigram: "th".to_string(),
            ms: 500.0,
            timestamp: 1,
        },
        BiometricSample {
            bigram: "th".to_string(),
            ms: 500.0,
            timestamp: 2,
        },
        BiometricSample {
            bigram: "th".to_string(),
            ms: 500.0,
            timestamp: 3,
        },
        BiometricSample {
            bigram: "th".to_string(),
            ms: 500.0,
            timestamp: 4,
        },
    ];

    let builder_bio = SessionBuilder::new(&loader)
        .with_keyboard_def(kb_def)
        .with_cost_model_obj(cost_model)
        .with_corpus_obj(corpus_arc)
        .with_biometrics(biometrics);
    let session_bio = builder_bio.build().unwrap();
    let engine_bio = session_bio.engine;

    let mut keys = vec![KeyCode(0); 30];
    keys[0] = KeyCode(116);
    keys[1] = KeyCode(104);
    let layout = Layout::new_unchecked(keys);

    let score_none = engine_none.score(&layout).unwrap();
    let score_bio = engine_bio.score(&layout).unwrap();

    println!("Score without Biometrics: {:.4}", score_none.to_f32());
    println!("Score with High Latency 'th': {:.4}", score_bio.to_f32());

    if score_bio > score_none {
        println!("SUCCESS: Biometrics affected the score!");
    } else {
        println!("FAILURE: Biometrics had no effect.");
        std::process::exit(1);
    }
}
