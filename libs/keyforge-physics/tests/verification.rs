// libs/keyforge-physics/tests/verification.rs

//! Integration tests for physics-based layout verification. Uses property-based testing
//! (`proptest`) to ensure parity between the optimized `ScoringEngine` and a deterministic
//! oracle, and validates the mathematical correctness of incremental swap-delta calculations
//! compared to full layout re-scores.


use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, 
    types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}
};
use keyforge_physics::{verify::DeterministicScorer, ScoringEngine};
use proptest::prelude::*;
use rand::SeedableRng;
use std::sync::Arc;

// --- STRATEGIES ---

fn rubric_strategy() -> impl Strategy<Value = Rubric> {
    (
        0.0..1000.0f32,                     // sfb_base
        0.0..500.0f32,                      // sfb_lateral
        0.0..10.0f32,                       // travel_lat
        0.0..5.0f32,                        // travel_vert
        prop::array::uniform5(0.0..5.0f32), // finger_effort
        0.0..200.0f32,                      // redirect
        0.0..100.0f32,                      // roll_bonus
    )
    .prop_map(|(sfb, sfb_lat, t_lat, t_vert, fingers, redir, roll)| Rubric {
        sfb_base: sfb,
        sfb_lateral: sfb_lat,
        travel_lat: t_lat,
        travel_vert: t_vert,
        finger_effort: fingers,
        redirect: redir,
        roll_bonus: roll,
        trigram_coverage: 1.0, // Full coverage for deterministic check
        trigram_limit: 100_000,
        ..Default::default()
    })
}

// Generates a Keyboard and a matching Layout of the same size
fn kb_and_layout_strategy() -> impl Strategy<Value = (Keyboard, Vec<KeyCode>)> {
    (10..50usize).prop_flat_map(|count| {
        let kb_strat = prop::collection::vec(
            (
                -20.0..20.0f32, // x
                -20.0..20.0f32, // y
                0u8..2,         // hand
                0u8..5,         // finger
                -5i8..5,        // row
                -10i8..15,      // col
            ),
            count,
        )
        .prop_map(move |keys_data| {
            let keys = keys_data
                .into_iter()
                .enumerate()
                .map(|(i, (x, y, hand, finger, row, col))| KeyNode {
                    index: i,
                    label: format!("k{}", i),
                    hand: HandIndex(hand),
                    finger: FingerIndex(finger),
                    row: RowIndex(row),
                    col: ColIndex(col),
                    x,
                    y,
                    is_home: row == 1,
                    ..Default::default()
                })
                .collect();
            Keyboard::new(keys, 1).unwrap()
        });

        let layout_strat = prop::collection::hash_set(0u16..255, count)
            .prop_map(|codes| codes.into_iter().map(KeyCode).collect::<Vec<_>>());

        (kb_strat, layout_strat)
    })
}

fn corpus_strategy(char_range: std::ops::Range<u16>) -> impl Strategy<Value = Corpus> {
    (
        prop::collection::vec((char_range.clone(), char_range.clone(), 1u32..1000), 0..20),
        prop::collection::vec((char_range.clone(), char_range.clone(), char_range.clone(), 1u32..1000), 0..20),
        prop::collection::vec(0u64..1000, 256),
    )
    .prop_map(|(bigrams, trigrams, char_freqs)| {
        let mut c = Corpus::default();
        c.bigrams = bigrams;
        c.trigrams = trigrams;
        c.char_freqs = char_freqs;
        c
    })
}

// --- TESTS ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_oracle_parity(
        (kb, layout_keys) in kb_and_layout_strategy(),
        corpus in corpus_strategy(0..255),
        rubric in rubric_strategy()
    ) {
        let layout = Layout::new_unchecked(layout_keys);
        let cost_matrix = vec![];
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();

        let fast_score = engine.score(&layout).unwrap();
        let slow_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);

        let diff = (fast_score - slow_score).abs();
        // Allow epsilon for float conversion noise and implementation differences
        let tolerance = (fast_score.abs() * 1e-3).max(0.1);

        prop_assert!(
            diff < tolerance,
            "Divergence! Fast: {}, Oracle: {}, Diff: {} (Allowed: {})",
            fast_score, slow_score, diff, tolerance
        );
    }

    #[test]
    fn test_delta_validity(
        (kb, mut layout_keys) in kb_and_layout_strategy(),
        cp in corpus_strategy(0..30),
        seed in any::<u64>(),
        swap_idx_1 in 0..100usize, // Will modulo later
        swap_idx_2 in 0..100usize
    ) {
        let len = layout_keys.len();
        if len < 2 { return Ok(()); }
        
        let i = swap_idx_1 % len;
        let j = swap_idx_2 % len;
        if i == j { return Ok(()); }

        // Ensure unique layout for clean delta testing
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        use rand::seq::SliceRandom;
        layout_keys.shuffle(&mut rng);

        let rubric = Rubric::default();
        let cost_matrix = vec![];
        let engine = ScoringEngine::new(&Arc::new(kb), &Arc::new(cp), &Arc::new(rubric), &cost_matrix).unwrap();

        let score_before = engine.score_raw(&layout_keys).unwrap();
        
        let mut pos_map = vec![65535u16; 65536];
        for (idx, &code) in layout_keys.iter().enumerate() {
            pos_map[code.0 as usize] = idx as u16;
        }

        let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, i, j).unwrap();

        layout_keys.swap(i, j);
        let score_after = engine.score_raw(&layout_keys).unwrap();
        
        let actual_delta = score_after - score_before;

        prop_assert_eq!(
            actual_delta, delta, 
            "Delta mismatch! Actual: {}, Calculated: {}", 
            actual_delta, delta
        );
    }
}

#[test]
fn test_delta_internals_manual() {
    // Deterministic test for Reverse Bigram and Trigram Middle logic
    let keys = vec![
        KeyNode { index: 0, x: 0.0, ..Default::default() },
        KeyNode { index: 1, x: 10.0, ..Default::default() },
        KeyNode { index: 2, x: 20.0, ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();
    
    let mut corpus = Corpus::default();
    // A->B (0->1). Swapping B affects this (Reverse Bigram path)
    corpus.bigrams.push((0, 1, 100));
    // X->Y->Z (0->1->2). Swapping Y affects this (Trigram Middle path)
    corpus.trigrams.push((0, 1, 2, 100));
    
    let mut rubric = Rubric::default();
    rubric.travel_lat = 1.0;
    
    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();
    
    // Layout: A=K0, B=K1, C=K2
    let mut layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
    let mut pos_map = vec![65535u16; 65536];
    pos_map[0] = 0; pos_map[1] = 1; pos_map[2] = 2;
    
    let score_before = engine.score_raw(&layout_keys).unwrap();
    
    // Swap B (K1) with C (K2).
    // B moves from 10.0 to 20.0.
    // Bigram A->B: Dist 10 -> 20. Cost increases.
    // Trigram A->B->C: Dist 10+10 -> 20+10? (Depends on flow logic)
    let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, 1, 2).unwrap();
    
    layout_keys.swap(1, 2);
    let score_after = engine.score_raw(&layout_keys).unwrap();
    
    assert_eq!(score_after - score_before, delta, "Manual delta check failed");
}

#[test]
fn test_delta_self_loop() {
    let keys = vec![
        KeyNode { index: 0, x: 0.0, ..Default::default() },
        KeyNode { index: 1, x: 10.0, ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();
    
    let mut corpus = Corpus::default();
    // A->A (0->0). Self loop.
    corpus.bigrams.push((0, 0, 100));
    
    let mut rubric = Rubric::default();
    rubric.travel_lat = 1.0;
    
    // Explicitly empty trigrams to force incremental path
    rubric.trigram_limit = 0; 
    
    let cost_matrix = vec![];
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_matrix).unwrap();
    
    // Layout: A=K0, B=K1
    let mut layout_keys = vec![KeyCode(0), KeyCode(1)];
    let mut pos_map = vec![65535u16; 65536];
    pos_map[0] = 0; pos_map[1] = 1;
    
    let score_before = engine.score_raw(&layout_keys).unwrap();
    
    // Swap A and B. A moves 0->1. B moves 1->0.
    // Bigram A->A:
    // Old: K0->K0 (Dist 0). Cost 0.
    // New: K1->K1 (Dist 0). Cost 0.
    // Delta should be 0.
    
    // If bug exists:
    // Old: K0->K0
    // New: K1->K0 ? (Dist 10).
    
    let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, 0, 1).unwrap();
    
    layout_keys.swap(0, 1);
    let score_after = engine.score_raw(&layout_keys).unwrap();
    
    assert_eq!(score_after - score_before, delta, "Self loop delta check failed");
}