use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::ScorerBuildParams; // FIXED
use std::io::Cursor;

fn setup_consistency_env() -> (keyforge_core::scorer::Scorer, Vec<u16>) {
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

    let ngram_data = "a\t100\nb\t100\nab\t50";
    let cost_data = "From,To,Cost\nk1,k2,10.0";

    // FIXED: Use ScorerBuildParams
    let scorer = ScorerBuildParams::from_readers(
        Cursor::new(cost_data),
        Cursor::new(ngram_data),
        geom,
        Some(ScoringWeights::default()),
        None,
        false,
    )
    .expect("Failed to build scorer");

    let layout = vec![b'a' as u16, b'b' as u16];
    (scorer, layout)
}

#[test]
fn test_scorer_engine_consistency() {
    let (scorer, layout) = setup_consistency_env();
    let pos_map = mutation::build_pos_map(&layout);

    let (fast_score, _, _) = scorer.score_full(&pos_map, 100);
    let details = scorer.score_details(&pos_map, 100);

    println!("Fast: {}, Detailed: {}", fast_score, details.layout_score);

    let diff = (fast_score - details.layout_score).abs();
    assert!(
        diff < 0.001,
        "Scoring Engines Diverged! Fast: {}, Detailed: {}",
        fast_score,
        details.layout_score
    );
}
