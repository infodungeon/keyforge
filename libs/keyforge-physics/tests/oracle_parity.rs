use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
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
                trigram_limit: 1000,
            },
        )
}

fn keyboard_strategy() -> impl Strategy<Value = Keyboard> {
    // Generate 10 to 50 keys
    (10..50usize).prop_flat_map(|count| {
        prop::collection::vec(
            (
                0.0..20.0f32, // x
                0.0..10.0f32, // y
                0u8..2,       // hand
                0u8..5,       // finger
                0i8..4,       // row
                0i8..12,      // col
            ),
            count,
        )
        .prop_map(move |keys_data| {
            let keys = keys_data
                .into_iter()
                .enumerate()
                .map(|(i, (x, y, hand, finger, row, col))| KeyNode {
                    id: i,
                    label: format!("k{}", i),
                    hand,
                    finger,
                    row,
                    col,
                    x,
                    y,
                    is_home: row == 1, // Dummy logic
                })
                .collect();
            Keyboard::new(keys, 1)
        })
    })
}

fn corpus_strategy() -> impl Strategy<Value = Corpus> {
    // Generate random bigrams and trigrams
    (
        prop::collection::vec((0u16..255, 0u16..255, 1u32..1000), 0..50),
        prop::collection::vec((0u16..255, 0u16..255, 0u16..255, 1u32..1000), 0..50),
    )
        .prop_map(|(bigrams, trigrams)| {
            let mut c = Corpus::default();
            c.bigrams = bigrams;
            c.trigrams = trigrams;
            c
        })
}

// --- THE ORACLE TEST ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_engine_matches_oracle(
        kb in keyboard_strategy(),
        corpus in corpus_strategy(),
        rubric in rubric_strategy(),
        seed in any::<u64>()
    ) {
        // 1. Setup Engines
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);

        // 2. Generate Random Layout
        let key_count = kb.count();
        let mut keys: Vec<u16> = (0..key_count as u16).collect();

        // Shuffle deterministically based on seed
        let mut rng = fastrand::Rng::with_seed(seed);
        rng.shuffle(&mut keys);

        let layout = Layout::new(keys);

        // 3. Run Shadow Execution
        let fast_score = engine.score(&layout);
        let slow_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout);

        // 4. Assert Parity
        let diff = (fast_score - slow_score).abs();

        // Tolerance Calculation:
        // Allow relative error of 0.001% (1e-5) or absolute 0.1, whichever is larger.
        let tolerance = (fast_score.abs() * 0.00001).max(0.1);

        prop_assert!(
            diff < tolerance,
            "Divergence! Fast: {}, Oracle: {}, Diff: {} (Allowed: {})",
            fast_score, slow_score, diff, tolerance
        );
    }
}
