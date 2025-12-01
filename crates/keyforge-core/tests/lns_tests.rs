use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::{mutation, Replica};
use keyforge_core::scorer::ScorerBuilder;
use std::io::Cursor;
use std::sync::Arc;

// --- Setup Helper ---
fn setup_simple_env() -> Arc<keyforge_core::scorer::Scorer> {
    // 1. Geometry: 5 Keys in a row (all on left hand, finger 1)
    let mut keys = Vec::new();
    for i in 0..5 {
        keys.push(KeyNode {
            id: format!("k{}", i),
            hand: 0,
            finger: 1,
            row: 0,
            col: i as i8,
            x: (i * 10) as f32,
            y: 0.0,
            is_stretch: false,
        });
    }

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![0], // Only k0 is prime
        med_slots: vec![1, 2],
        low_slots: vec![3, 4],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    // 2. Data: 'a' is very frequent, 'b' is medium, 'c' is rare
    let mut ngram_data = String::new();
    ngram_data.push_str("a\t1000\n");
    ngram_data.push_str("b\t100\n");
    ngram_data.push_str("c\t10\n");

    let cursor = Cursor::new(ngram_data);

    // 3. Weights
    let weights = ScoringWeights {
        weight_lateral_travel: 1.0,
        // Using standard scale, but we must ensure consistent calculation
        ..Default::default()
    };

    // 4. Build
    ScorerBuilder::new()
        .with_weights(weights)
        .with_defs(LayoutDefinitions::default())
        .with_geometry(geom)
        .with_ngrams_from_reader(cursor)
        .unwrap()
        .build()
        .map(Arc::new)
        .unwrap()
}

#[test]
fn test_lns_finds_optimum() {
    let scorer = setup_simple_env();

    let mut replica = Replica::new(
        scorer.clone(),
        0.0, // Zero temp -> Greedy
        Some(123),
        false,
        100,
        100,
        "",
    );

    // Force specific layout: [c, b, a, 0, 0]
    replica.layout = vec![b'c', b'b', b'a', 0, 0];
    replica.pos_map = mutation::build_pos_map(&replica.layout);

    // FIX: Update internal state in the correct order!
    // 1. Get raw physics metrics
    let (initial_score, l, t) = scorer.score_full(&replica.pos_map, 100);

    // 2. Update Replica State FIRST
    replica.left_load = l;
    replica.total_freq = t;

    // 3. Calculate Total Score (using the NOW CORRECT total_freq)
    replica.score = initial_score + replica.imbalance_penalty(l);

    // 4. Update Weights
    replica.update_mutation_weights();

    println!("Initial Score: {}", replica.score);

    // Run LNS on 3 keys
    let mut improved = false;
    // With 100 tries, uniform weights (approx) from Cost-Guided logic
    // on this small set should definitely find the swap.
    for _ in 0..100 {
        if replica.try_lns_move(3) {
            improved = true;
            break;
        }
    }

    println!("Final Score: {}", replica.score);

    assert!(
        improved,
        "LNS failed to improve a clearly suboptimal layout"
    );
    assert!(
        replica.score < 100.0 + initial_score,
        "Score should decrease or stay constrained"
    );

    // Validate final positions
    // 'a' (1000 freq) should be at index 0 (dist 0)
    assert_eq!(replica.layout[0], b'a', "'a' did not move to prime slot");
}

#[test]
fn test_lns_respects_locks() {
    let scorer = setup_simple_env();
    let pin_config = "0:c";

    let mut replica = Replica::new(scorer.clone(), 0.0, Some(123), false, 100, 100, pin_config);

    replica.layout = vec![b'c', b'b', b'a', 0, 0];
    replica.pos_map = mutation::build_pos_map(&replica.layout);

    // Correct update order here too
    let (initial_score, l, t) = scorer.score_full(&replica.pos_map, 100);
    replica.left_load = l;
    replica.total_freq = t;
    replica.score = initial_score + replica.imbalance_penalty(l);

    replica.update_mutation_weights();

    let mut improved = false;
    for _ in 0..100 {
        if replica.try_lns_move(3) {
            improved = true;
            break;
        }
    }

    assert!(
        improved,
        "LNS should still improve the remaining unpinned keys"
    );
    assert_eq!(replica.layout[0], b'c', "LNS moved a pinned key!");
    assert_eq!(
        replica.layout[1], b'a',
        "'a' should have moved to the best AVAILABLE spot (slot 1)"
    );
}
