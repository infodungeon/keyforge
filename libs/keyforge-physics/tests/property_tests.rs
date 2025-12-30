use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric, types::{HandIndex, FingerIndex, RowIndex, ColIndex, KeyCode}};
use keyforge_physics::{verify::DeterministicScorer, ScoringEngine};
use proptest::prelude::*;

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
        .prop_map(
            |(sfb, sfb_lat, t_lat, t_vert, fingers, redir, roll)| Rubric {
                sfb_base: sfb,
                sfb_lateral: sfb_lat,
                travel_lat: t_lat,
                travel_vert: t_vert,
                finger_effort: fingers,
                redirect: redir,
                roll_bonus: roll,
                trigram_coverage: 1.0, // Full coverage for deterministic check
                trigram_limit: 100_000,
            },
        )
}

fn keyboard_strategy() -> impl Strategy<Value = Keyboard> {
    // Generate 10 to 50 keys with potentially weird indices to test guardrails
    (10..50usize).prop_flat_map(|count| {
        prop::collection::vec(
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
        })
    })
}

fn corpus_strategy() -> impl Strategy<Value = Corpus> {
    // Generate random bigrams and trigrams
    (
        prop::collection::vec((0u16..255, 0u16..255, 1u32..1000), 0..20),
        prop::collection::vec((0u16..255, 0u16..255, 0u16..255, 1u32..1000), 0..20),
        prop::collection::vec(0u32..1000, 256),
    )
        .prop_map(|(bigrams, trigrams, char_freqs)| {
            let mut c = Corpus::default();
            c.bigrams = bigrams;
            c.trigrams = trigrams;
            c.char_freqs = char_freqs;
            c
        })
}

// --- THE ORACLE TEST ---

proptest! {
    // #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_engine_matches_oracle_strict(
        kb in keyboard_strategy(),
        corpus in corpus_strategy(),
        rubric in rubric_strategy(),
        // Generate a random layout of valid key indices
        mut layout_keys in prop::collection::vec(0u16..255, 50)
    ) {
        // Clamp layout keys to actual keyboard size
        let key_count = kb.count();
        if key_count == 0 { return Ok(()); }

        for k in layout_keys.iter_mut() {
            *k = *k % (key_count as u16);
        }
        let layout = Layout::new_unchecked(layout_keys.into_iter().map(KeyCode).collect());

        // 1. Setup Engines
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

        // 2. Run Shadow Execution
        let fast_score = engine.score(&layout).unwrap();
        let slow_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &[]);

        // 3. Assert Parity
        let diff = (fast_score - slow_score).abs();

        // Tolerance Calculation:
        // Use a relative epsilon for large scores, absolute for small ones.
        // fast_score is f32 result of i64 fixed-point math scaled down.
        // slow_score is f32 result of i64 fixed-point math scaled down.
        // They should be bitwise identical if the logic matches exactly,
        // but floating point conversion at the end might introduce epsilon noise.

        let tolerance = (fast_score.abs() * 1e-5).max(0.1);

        prop_assert!(
            diff < tolerance,
            "Divergence! Fast: {}, Oracle: {}, Diff: {} (Allowed: {})",
            fast_score, slow_score, diff, tolerance
        );
    }
}
