use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric};
use keyforge_physics::ScoringEngine;
use proptest::prelude::*;
use rand::SeedableRng;
use std::sync::Arc;

fn arb_keyboard(size: usize) -> impl Strategy<Value = Keyboard> {
    prop::collection::vec(
        (0..5u8, 0..3i8, 0..10i8).prop_map(|(f, r, c)| KeyNode {
            id: 0, // Placeholder
            label: "k".to_string(),
            hand: if c < 5 { 0 } else { 1 },
            finger: f,
            row: r,
            col: c,
            x: c as f32,
            y: r as f32,
            is_home: false,
        }),
        size,
    )
    .prop_map(|mut nodes| {
        for (i, node) in nodes.iter_mut().enumerate() {
            node.id = i;
        }
        Keyboard::new(nodes, 1)
    })
}

fn arb_corpus(char_range: std::ops::Range<u16>) -> impl Strategy<Value = Corpus> {
    let bigrams =
        prop::collection::vec((char_range.clone(), char_range.clone(), 0..1000u32), 0..20);
    let trigrams = prop::collection::vec(
        (
            char_range.clone(),
            char_range.clone(),
            char_range.clone(),
            0..500u32,
        ),
        0..20,
    );
    let char_freqs = prop::collection::vec(0..100u32, (char_range.end - char_range.start) as usize);

    (char_freqs, bigrams, trigrams).prop_map(move |(freqs, bigs, tris)| {
        let mut corpus = Corpus::default();
        for (i, &f) in freqs.iter().enumerate() {
            corpus.char_freqs[char_range.start as usize + i] = f;
        }
        corpus.bigrams = bigs;
        corpus.trigrams = tris;
        corpus
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_delta_parity_unique_keys(
        kb in arb_keyboard(30),
        cp in arb_corpus(0..30),
        mut layout_keys in prop::collection::vec(0..30u16, 30).prop_map(|_| {
            // Use 0..30 to ensure we hit the chars in the corpus char_range
            (0..30).collect::<Vec<u16>>()
        }),
        seed in any::<u64>(),
        swap_idxs in (0..30usize, 0..30usize)
    ) {
        let (i, j) = swap_idxs;
        if i == j { return Ok(()); }

        // Manually shuffle since we want a random but unique layout
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        use rand::seq::SliceRandom;
        layout_keys.shuffle(&mut rng);

        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&Arc::new(kb), &Arc::new(cp), &Arc::new(rubric), &[]).unwrap();

        let score_before = engine.score_raw(&layout_keys);

        // Precompute pos_map for delta
        let mut pos_map = vec![65535u16; 65536];
        for (idx, &code) in layout_keys.iter().enumerate() {
            pos_map[code as usize] = idx as u16;
        }

        let delta = engine.calculate_swap_delta(&layout_keys, &pos_map, i, j);

        // Apply swap
        layout_keys.swap(i, j);

        let score_after = engine.score_raw(&layout_keys);
        let actual_delta = score_after - score_before;

        prop_assert_eq!(actual_delta, delta, "Delta mismatch! score_after({}) - score_before({}) = {}, but calculate_swap_delta returned {}", score_after, score_before, actual_delta, delta);
    }
}
