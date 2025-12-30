use keyforge_adapter::conversion;
use keyforge_core::ScoringEngine;
use keyforge_infra::FsProvider;
use keyforge_infra::AssetLoader;
use keyforge_protocol::config::{CorpusSource, ScoringWeights};
use std::path::PathBuf;

#[test]
fn test_real_data_pipeline() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = PathBuf::from(manifest_dir).join("../../data");

    if !root.exists() {
        println!(
            "Skipping real data test: 'data' directory not found at {:?}",
            root
        );
        return;
    }

    println!("Loading real data from: {:?}", root);

    let provider = FsProvider::new(root.clone());

    // Construct Weights manually since defaults are zero
    let weights = ScoringWeights {
        penalty_sfb_base: 400.0,
        penalty_scissor: 25.0,
        penalty_redirect: 65.0,
        weight_vertical_travel: 1.0,
        weight_lateral_travel: 3.5,
        finger_penalty_scale: "0.0,1.0,1.1,1.3,1.6".to_string(),
        ..Default::default()
    };

    let sources = [CorpusSource {
        id: "text/en_std".into(),
        weight: 1.0,
        hash: None,
    }];

    // Load real assets directly
    let def = provider
        .load_keyboard("ansi_104")
        .expect("Failed to load keyboard");
    let corpus = provider
        .load_corpus(&sources)
        .expect("Failed to load corpus");
    let cost_data = provider
        .load_cost_matrix("default_costmatrix.json")
        .expect("Failed to load cost matrix");

    let rubric = conversion::to_domain_rubric(&weights);
    let keyboard = conversion::to_domain_keyboard(&def.geometry);
    let cost_overrides = cost_data.resolve(&def.geometry);

    let engine = ScoringEngine::new(&keyboard, &corpus, &rubric, &cost_overrides)
        .expect("Failed to create scoring engine");

    let registry = provider
        .load_keycodes("keycodes.json")
        .expect("Failed to load keycodes");

    // Analyze a concrete layout string (similar to CLI/UI input)
    let qwerty = "Q W E R T Y U I O P A S D F G H J K L Z X C V B N M";
    let layout = conversion::parse_layout_string(qwerty, engine.key_count(), &registry)
        .expect("Failed to parse layout");

    let report = engine.analyze(&layout);

    println!("Score: {}", report.score);
    println!("Distance: {}", report.distance);
    println!("SFB Total: {}", report.sfb_total);

    // Assertions
    assert!(report.score > 0.0, "Score should be non-zero");
    assert!(report.distance > 0.0, "Distance should be non-zero");

    // Verify Heatmap
    assert!(!report.heatmap.is_empty(), "Heatmap should be populated");
    let max_heat = report.heatmap.iter().cloned().fold(f32::NAN, f32::max);
    assert!(max_heat > 0.0, "Heatmap should have non-zero values");
}
