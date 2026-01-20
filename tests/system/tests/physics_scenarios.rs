// tests/system/tests/physics_scenarios.rs

use keyforge_adapter::conversion;
use keyforge_physics::EngineFactory;
use keyforge_infra::{AssetLoader, FsProvider};
use keyforge_model::config::{CorpusSource, ScoringWeights};
use keyforge_model::constants::{ASSET_COST_MATRIX, ASSET_KEYCODES};
use keyforge_model::{CostModel, Keyboard, KeyboardDefinition, KeycodeRegistry};
use std::path::PathBuf;

fn get_data_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../../data")
        .canonicalize()
        .expect("Failed to find data dir")
}

#[tokio::test]
async fn test_scorer_determinism_production_data() {
    let data_dir = get_data_dir();
    let kb_path = data_dir.join("system/keyboards/models/corne.mpk.zst");

    if !data_dir
        .join("system/weights")
        .join("cost_matrix.mpk.zst")
        .exists()
        || !kb_path.exists()
    {
        println!("⚠️ Skipping parity test: Production data missing.");
        return;
    }

    let sources = vec![CorpusSource {
        id: "text/en_std".to_string(),
        weight: 1.0,
        hash: None,
    }];

    let provider = FsProvider::new(data_dir.clone());

    let corpus = provider
        .load_corpus(&sources)
        .await
        .expect("Failed to load corpus");
    let cost_data = provider
        .load::<CostModel>(ASSET_COST_MATRIX)
        .await
        .expect("Failed to load cost matrix");
    let def = provider
        .load::<KeyboardDefinition>("corne")
        .await
        .expect("Failed to load keyboard");

    let weights = ScoringWeights::default();
    let keyboard = Keyboard::new(def.geometry.keys.clone(), def.geometry.home_row).unwrap();
    let rubric = conversion::to_domain_rubric(&weights);

    let engine = EngineFactory::new_generic(&keyboard, &corpus, &rubric, &cost_data)
        .expect("Failed to create engine");

    let registry = provider
        .load::<KeycodeRegistry>(ASSET_KEYCODES)
        .await
        .expect("Failed to load keycodes");
    let qwerty = "Q W E R T Y U I O P A S D F G H J K L Z X C V B N M";
    let layout = conversion::parse_layout_string(qwerty, engine.key_count(), &registry).unwrap();

    let report1 = engine.analyze(&layout).unwrap();
    let report2 = engine.analyze(&layout).unwrap();
    let score1 = engine.score(&layout).unwrap();
    let score2 = engine.score(&layout).unwrap();

    assert!(
        (report1.score - report2.score).abs() < 0.001,
        "Report scores diverged!"
    );
    assert_eq!(score1, score2, "Engine scores diverged!");
}
