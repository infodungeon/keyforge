// ===== keyforge/crates/keyforge-core/tests/simulation_test.rs =====
use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::scorer::ScorerBuildParams;
use tempfile::tempdir;

#[test]
fn test_simulation_flow() {
    let mut keys = Vec::new();
    for i in 0..10 {
        keys.push(KeyNode {
            id: format!("k{}", i),
            hand: 0,
            finger: 1,
            row: 0,
            col: i as i8,
            x: i as f32,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        });
    }
    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    let dir = tempdir().unwrap();
    let cost_path = dir.path().join("cost.csv");
    let corpus_dir = dir.path().join("corpus");
    std::fs::create_dir(&corpus_dir).unwrap();

    std::fs::write(&cost_path, "From,To,Cost\n").unwrap();
    std::fs::write(corpus_dir.join("1grams.csv"), "c,f\na,100").unwrap();
    std::fs::write(corpus_dir.join("2grams.csv"), "c1,c2,f\n").unwrap();
    std::fs::write(corpus_dir.join("3grams.csv"), "c1,c2,c3,f\n").unwrap();

    let scorer = ScorerBuildParams::load_from_disk(
        cost_path,
        corpus_dir,
        geom,
        Some(ScoringWeights::default()),
        None,
        false,
    )
    .expect("Failed to build scorer");

    assert_eq!(scorer.key_count, 10);
}
