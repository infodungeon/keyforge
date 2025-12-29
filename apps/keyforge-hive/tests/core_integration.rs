use keyforge_adapter::conversion;
use keyforge_core::ScoringEngine;
use keyforge_infra::AssetLoader;
use keyforge_infra::FsProvider;
use keyforge_protocol::config::{CorpusSource, ScoringWeights};
use keyforge_protocol::geometry::KeyboardDefinition;
use std::env;
use std::path::PathBuf;

fn get_data_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../../data")
        .canonicalize()
        .unwrap()
}

#[test]
fn test_scorer_determinism_production_data() {
    let data_dir = get_data_dir();
    let cost_path = "cost_matrix.json";
    let kb_path = data_dir.join("keyboards/corne.json");

    if !data_dir.join(cost_path).exists() || !kb_path.exists() {
        println!("⚠️ Skipping parity test: Production data missing.");
        return;
    }

    let content = std::fs::read_to_string(&kb_path).unwrap();
    let def: KeyboardDefinition = serde_json::from_str(&content).unwrap();

    let sources = vec![CorpusSource {
        id: "text/en_std".to_string(),
        weight: 1.0,
        hash: None,
    }];

    // Load everything from disk twice to test determinism across identical rebuilds.
    let provider_a = FsProvider::new(data_dir.clone());
    let provider_b = FsProvider::new(data_dir.clone());

    let corpus_a = provider_a
        .load_corpus(&sources)
        .expect("Failed to load corpus");
    let corpus_b = provider_b
        .load_corpus(&sources)
        .expect("Failed to load corpus");

    let cost_data_a = provider_a
        .load_cost_matrix(cost_path)
        .expect("Failed to load cost matrix");
    let cost_data_b = provider_b
        .load_cost_matrix(cost_path)
        .expect("Failed to load cost matrix");

    let weights = ScoringWeights::default();
    let keyboard_a = conversion::to_domain_keyboard(&def.geometry);
    let keyboard_b = conversion::to_domain_keyboard(&def.geometry);
    let rubric_a = conversion::to_domain_rubric(&weights);
    let rubric_b = conversion::to_domain_rubric(&weights);

    let overrides_a = conversion::resolve_cost_matrix(&cost_data_a.entries, &def.geometry);
    let overrides_b = conversion::resolve_cost_matrix(&cost_data_b.entries, &def.geometry);

    let engine_a = ScoringEngine::new(&keyboard_a, &corpus_a, &rubric_a, &overrides_a)
        .expect("Failed to create engine A");
    let engine_b = ScoringEngine::new(&keyboard_b, &corpus_b, &rubric_b, &overrides_b)
        .expect("Failed to create engine B");

    // Create Layout String (Space separated tokens)
    let qwerty = "Q W E R T Y U I O P A S D F G H J K L Z X C V B N M";

    println!("🧪 Calculating Scores...");

    // Parse layout once (using keycodes from disk)
    let registry = provider_a
        .load_keycodes("keycodes.json")
        .expect("Failed to load keycodes");

    let layout = conversion::parse_layout_string(qwerty, engine_a.key_count(), &registry).unwrap();

    let report_a = engine_a.analyze(&layout);
    let report_b = engine_b.analyze(&layout);

    println!("   Score A: {:.6}", report_a.score);
    println!("   Score B: {:.6}", report_b.score);

    let diff = (report_a.score - report_b.score).abs();

    assert!(diff < 0.001, "Scores diverged! Diff: {}", diff);

    println!("✅ Parity Check Passed: The Scorer is Deterministic.");
}
