// ===== keyforge/tests/dynamic_size_tests.rs =====
use keyforge::config::Config;
use keyforge::geometry::{KeyNode, KeyboardGeometry};
use std::fs::File;
use std::io::Write;

#[test]
fn test_scorer_handles_small_geometry() {
    // 1. Setup 3-Key Geometry
    let keys = vec![
        KeyNode {
            id: "k1".to_string(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            is_stretch: false,
        },
        KeyNode {
            id: "k2".to_string(),
            hand: 0,
            finger: 2,
            row: 0,
            col: 1,
            x: 1.0,
            y: 0.0,
            is_stretch: false,
        },
        KeyNode {
            id: "k3".to_string(),
            hand: 0,
            finger: 3,
            row: 0,
            col: 2,
            x: 2.0,
            y: 0.0,
            is_stretch: false,
        },
    ];

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![0, 1, 2],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    // 2. Mock Data Files
    let dir = tempfile::tempdir().unwrap();
    let cost_path = dir.path().join("small_cost.csv");
    let ngram_path = dir.path().join("small_ngrams.tsv");

    {
        let mut f = File::create(&cost_path).unwrap();
        writeln!(f, "From,To,Cost").unwrap();
    }

    {
        let mut f = File::create(&ngram_path).unwrap();
        writeln!(f, "ab\t100").unwrap(); // Bigram A-B
    }

    // 3. Initialize Scorer
    let mut config = Config::default();
    config.defs.tier_high_chars = "abc".to_string(); // Only 3 chars needed

    let scorer_res = keyforge::scorer::setup::build_scorer(
        cost_path.to_str().unwrap(),
        ngram_path.to_str().unwrap(),
        config.weights,
        config.defs,
        geom.clone(),
        true,
    );

    assert!(
        scorer_res.is_ok(),
        "Scorer failed to initialize with 3 keys"
    );
    let scorer = scorer_res.unwrap();

    assert_eq!(scorer.key_count, 3);
    // 3x3 flat matrix = 9 elements
    assert_eq!(scorer.full_cost_matrix.len(), 9);

    // 4. Score a Layout
    // A=0, B=1, C=2
    let mut pos_map = [255u8; 256];
    pos_map[b'a' as usize] = 0;
    pos_map[b'b' as usize] = 1;
    pos_map[b'c' as usize] = 2;

    let (score, _, _) = scorer.score_full(&pos_map, 100);

    assert!(
        score > 0.0,
        "Score should be calculated (geometric distance cost)"
    );
}
