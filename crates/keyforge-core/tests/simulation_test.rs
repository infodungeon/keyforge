// ===== keyforge/crates/keyforge-core/tests/simulation_test.rs =====
use keyforge_core::config::{Config, LayoutDefinitions, ScoringWeights, SearchParams};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::{OptimizationOptions, Optimizer, ProgressCallback};
use keyforge_core::scorer::ScorerBuildParams;
use std::io::Cursor;
use std::sync::Arc;

struct TestLogger;
impl ProgressCallback for TestLogger {
    fn on_progress(&self, ep: usize, score: f32, _layout: &[u16], ips: f32) -> bool {
        println!("Test Ep: {} | Score: {:.2} | IPS: {:.2}", ep, score, ips);
        true
    }
}

#[test]
fn test_verify_system_simulation() {
    // 1. Replicate verify_system.py Geometry (10 keys, 5 fingers, rotated)
    let mut keys = Vec::new();
    for i in 0..10 {
        keys.push(KeyNode {
            id: format!("k{}", i),
            hand: if i < 5 { 0 } else { 1 },
            finger: (i % 5) as u8, // Unique finger rotation 0-4
            row: 0,
            col: i as i8,
            x: i as f32,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        });
    }

    let geom = KeyboardGeometry {
        keys,
        prime_slots: (0..10).collect(),
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };

    // 2. Replicate verify_system.py Weights & Params
    // FIXED: Use struct update syntax
    let weights = ScoringWeights {
        loader_trigram_limit: 1000,
        ..Default::default()
    };

    // 3. Mock Data
    let cost_csv = "From,To,Cost\nk0,k1,10.0\n";
    let ngram_tsv = "a\t100\nb\t100\nc\t100\nd\t100\ne\t100\nf\t100\n";

    // 4. Build Scorer
    let scorer = ScorerBuildParams::from_readers(
        Cursor::new(cost_csv),
        Cursor::new(ngram_tsv),
        geom,
        Some(weights.clone()),
        Some(LayoutDefinitions::default()),
        true, // Debug
    )
    .expect("Failed to build scorer");

    let scorer_arc = Arc::new(scorer);

    // 5. Build Optimizer Options
    let params = SearchParams {
        search_epochs: 5, // Short run
        search_steps: 100,
        search_patience: 5,
        ..Default::default()
    };

    let config = Config {
        search: params,
        weights,
        ..Default::default()
    };

    let mut options = OptimizationOptions::from(&config);
    options.num_threads = 1; // Single thread for determinism in test

    // 6. Run
    println!("🚀 Starting Simulation...");
    let optimizer = Optimizer::new(scorer_arc, options);
    let result = optimizer.run(Some(42), TestLogger);

    println!("🏁 Result Score: {}", result.score);
    assert!(result.score > 0.0);
    assert_eq!(result.layout.len(), 10);
}
