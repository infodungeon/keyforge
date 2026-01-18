// apps/keyforge-hive/tests/integration_assets.rs

//! Integration tests for Hive asset management and scoring parity.

use keyforge_adapter::conversion;
use keyforge_core::ScoringEngine;
use keyforge_infra::FsProvider;
use keyforge_infra::AssetLoader;
use keyforge_model::config::{CorpusSource, ScoringWeights};
use keyforge_model::constants::{ASSET_COST_MATRIX, ASSET_KEYCODES};
use keyforge_model::{Keyboard, KeyboardDefinition, CostModel, KeycodeRegistry};
use std::env;
use std::path::PathBuf;

fn get_data_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("../../data").canonicalize().unwrap()
}

#[tokio::test]
async fn test_scorer_determinism_production_data() {
    let data_dir = get_data_dir();
    let kb_path = data_dir.join("system/keyboards/models/corne.mpk.zst");

    if !data_dir.join("system/weights").join("cost_matrix.mpk.zst").exists() || !kb_path.exists() {
        println!("⚠️ Skipping parity test: Production data missing.");
        return;
    }

    let sources = vec![CorpusSource { id: "text/en_std".to_string(), weight: 1.0, hash: None }];
    
    // Load twice to ensure determinism across rebuilds
    let provider_a = FsProvider::new(data_dir.clone());
    let provider_b = FsProvider::new(data_dir.clone());

    let corpus_a = provider_a.load_corpus(&sources).await.expect("Failed to load corpus");
    let corpus_b = provider_b.load_corpus(&sources).await.expect("Failed to load corpus");

    let cost_data_a = provider_a.load::<CostModel>(ASSET_COST_MATRIX).await.expect("Failed to load cost matrix");
    let cost_data_b = provider_b.load::<CostModel>(ASSET_COST_MATRIX).await.expect("Failed to load cost matrix");

    // Load Keyboard (using system path resolver internally)
    let def_a = provider_a.load::<KeyboardDefinition>("corne").await.expect("Failed to load keyboard");
    let def_b = provider_b.load::<KeyboardDefinition>("corne").await.expect("Failed to load keyboard");

    let weights = ScoringWeights::default();
    
    let keyboard_a = Keyboard::new(def_a.geometry.keys.clone(), def_a.geometry.home_row).unwrap();
    let keyboard_b = Keyboard::new(def_b.geometry.keys.clone(), def_b.geometry.home_row).unwrap();
    
    let rubric_a = conversion::to_domain_rubric(&weights);
    let rubric_b = conversion::to_domain_rubric(&weights);

    let engine_a = ScoringEngine::new(&keyboard_a, &corpus_a, &rubric_a, &cost_data_a).expect("Failed to create engine A");
    let engine_b = ScoringEngine::new(&keyboard_b, &corpus_b, &rubric_b, &cost_data_b).expect("Failed to create engine B");

    let registry = provider_a.load::<KeycodeRegistry>(ASSET_KEYCODES).await.expect("Failed to load keycodes");
    let qwerty = "Q W E R T Y U I O P A S D F G H J K L Z X C V B N M";
    let layout = conversion::parse_layout_string(qwerty, engine_a.key_count(), &registry).unwrap();

    let report_a = engine_a.analyze(&layout).unwrap();
    let report_b = engine_b.analyze(&layout).unwrap();

    let diff = (report_a.score - report_b.score).abs();
    assert!(diff < 0.001, "Scores diverged! Diff: {}", diff);
}