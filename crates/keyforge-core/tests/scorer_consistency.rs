use keyforge_core::config::ScoringWeights; // FIXED: Removed Config
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::ScorerBuilder;
use std::io::Cursor;

fn setup_consistency_env() -> (keyforge_core::scorer::Scorer, Vec<u16>) {
    // 1. Geometry (2 keys)
    let keys = vec![
        KeyNode {
            id: "k1".into(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        },
        KeyNode {
            id: "k2".into(),
            hand: 0,
            finger: 1,
            row: 0,
            col: 1,
            x: 1.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        },
    ];
    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    // 2. Data (A, B, AB)
    let ngram_data = "a\t100\nb\t100\nab\t50";
    let cost_data = "From,To,Cost\nk1,k2,10.0"; // Base user cost

    // 3. Builder
    let scorer = ScorerBuilder::new()
        .with_geometry(geom)
        .with_weights(ScoringWeights::default())
        .with_costs_from_reader(Cursor::new(cost_data))
        .unwrap()
        .with_ngrams_from_reader(Cursor::new(ngram_data))
        .unwrap()
        .build()
        .unwrap();

    let layout = vec![b'a' as u16, b'b' as u16];
    (scorer, layout)
}

#[test]
fn test_scorer_engine_consistency() {
    let (scorer, layout) = setup_consistency_env();
    let pos_map = mutation::build_pos_map(&layout);

    // 1. Fast Engine (Used by Optimizer)
    let (fast_score, _, _) = scorer.score_full(&pos_map, 100);

    // 2. Detail Engine (Used by UI/Reports)
    let details = scorer.score_details(&pos_map, 100);

    println!("Fast: {}, Detailed: {}", fast_score, details.layout_score);

    // 3. Assert Equality
    // Allow small float epsilon, but logic should be identical
    let diff = (fast_score - details.layout_score).abs();
    assert!(
        diff < 0.001,
        "Scoring Engines Diverged! Fast: {}, Detailed: {}",
        fast_score,
        details.layout_score
    );
}
