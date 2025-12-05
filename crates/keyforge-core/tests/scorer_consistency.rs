// ===== keyforge/crates/keyforge-core/tests/scorer_consistency.rs =====
use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::ScorerBuildParams;
use tempfile::tempdir;

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

    let dir = tempdir().unwrap();
    let cost_path = dir.path().join("cost.csv");
    let corpus_dir = dir.path().join("corpus");
    std::fs::create_dir(&corpus_dir).unwrap();

    std::fs::write(&cost_path, "From,To,Cost\nk1,k2,10.0").unwrap();
    std::fs::write(corpus_dir.join("1grams.csv"), "c,f\na,100\nb,100").unwrap();
    std::fs::write(corpus_dir.join("2grams.csv"), "c1,c2,f\na,b,50").unwrap();
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
