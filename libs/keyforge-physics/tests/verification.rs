// libs/keyforge-physics/tests/verification.rs

use keyforge_model::{
    Corpus, KeyNode, Keyboard, Layout, Rubric, CostModel,
    types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}
};
use keyforge_physics::ScoringEngine;
use proptest::prelude::*;
use rand::SeedableRng;
use std::sync::Arc;

fn mock_cost_model() -> CostModel {
    let json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0 },
                        "index": { "base": { "r0": 100.0 } },
                        "middle": { "base": { "r0": 100.0 } },
                        "ring": { "base": { "r0": 100.0 } },
                        "pinky": { "base": { "r0": 100.0 } }
                    }
                }
            }
        },
        "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
    }"#;
    serde_json::from_str(json).unwrap()
}

fn rubric_strategy() -> impl Strategy<Value = Rubric> {
    (
        0.0..1000.0f32,
        0.0..500.0f32,
        0.0..10.0f32,
        0.0..5.0f32,
        prop::array::uniform5(0.0..5.0f32),
        0.0..200.0f32,
        0.0..100.0f32,
    )
    .prop_map(|(sfb, sfb_lat, t_lat, t_vert, fingers, redir, roll)| Rubric {
        sfb_base: sfb,
        sfb_lateral: sfb_lat,
        travel_lat: t_lat,
        travel_vert: t_vert,
        finger_effort: fingers,
        redirect: redir,
        roll_bonus: roll,
        trigram_coverage: 1.0,
        trigram_limit: 100_000,
        ..Default::default()
    })
}

fn kb_and_layout_strategy() -> impl Strategy<Value = (Keyboard, Vec<KeyCode>)> {
    (10..50usize).prop_flat_map(|count| {
        let kb_strat = prop::collection::vec(
            (
                -20.0..20.0f32,
                -20.0..20.0f32,
                0u8..2,
                0u8..5,
                -5i8..5,
                -10i8..15,
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

        let layout_strat = prop::collection::vec(0u16..255, count)
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_oracle_parity(
        (kb, layout_keys) in kb_and_layout_strategy(),
        corpus in corpus_strategy(0..255),
        rubric in rubric_strategy()
    ) {
        let layout = Layout::new_unchecked(layout_keys);
        let cost_model = mock_cost_model();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_model).unwrap();

        // Smoke test: Ensure scoring doesn't panic
        let _fast_score = engine.score(&layout).unwrap();
        
        // Note: Oracle parity check disabled until DeterministicScorer is updated to support CostModel.
    }

    #[test]
    fn test_delta_validity(
        (kb, mut layout_keys) in kb_and_layout_strategy(),
        cp in corpus_strategy(0..30),
        seed in any::<u64>(),
        swap_idx_1 in 0..100usize,
        swap_idx_2 in 0..100usize
    ) {
        let len = layout_keys.len();
        if len < 2 { return Ok(()); }
        
        let i = swap_idx_1 % len;
        let j = swap_idx_2 % len;
        if i == j { return Ok(()); }

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        use rand::seq::SliceRandom;
        layout_keys.shuffle(&mut rng);

        let rubric = Rubric::default();
        let cost_model = mock_cost_model();
        let engine = ScoringEngine::new(&Arc::new(kb), &Arc::new(cp), &Arc::new(rubric), &cost_model).unwrap();

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
    let keys = vec![
        KeyNode { index: 0, x: 0.0, ..Default::default() },
        KeyNode { index: 1, x: 10.0, ..Default::default() },
        KeyNode { index: 2, x: 20.0, ..Default::default() },
    ];
    let kb = Keyboard::new(keys, 0).unwrap();
    
    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 100));
    corpus.trigrams.push((0, 1, 2, 100));
    
    let mut rubric = Rubric::default();
    rubric.travel_lat = 1.0;
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    
    let mut layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
    let mut pos_map = vec![65535u16; 65536];
    pos_map[0] = 0; pos_map[1] = 1; pos_map[2] = 2;
    
    let score_before = engine.score_raw(&layout_keys).unwrap();
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
    corpus.bigrams.push((0, 0, 100));
    
    let mut rubric = Rubric::default();
    rubric.travel_lat = 1.0;
    rubric.trigram_limit = 0; 
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
    
    let mut layout_keys = vec![KeyCode(0), KeyCode(1)];
    let mut pos_map = vec![65535u16; 65536];
    pos_map[0] = 0; pos_map[1] = 1;
    
    let score_before = engine.score_raw(&layout_keys).unwrap();
    let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, 0, 1).unwrap();
    
    layout_keys.swap(0, 1);
    let score_after = engine.score_raw(&layout_keys).unwrap();
    
    assert_eq!(score_after - score_before, delta, "Self loop delta check failed");
}
