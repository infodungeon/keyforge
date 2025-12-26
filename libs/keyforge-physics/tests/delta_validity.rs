use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric};
use keyforge_physics::ScoringEngine;

#[test]
fn test_delta_matches_full_score() {
    let keys = (0..30)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: if i < 15 { 0 } else { 1 },
            finger: (i % 5) as u8,
            row: (i / 10) as i8,
            col: (i % 10) as i8,
            x: (i % 10) as f32,
            y: (i / 10) as f32,
            is_home: (10..20).contains(&i),
        })
        .collect();

    let keyboard = Keyboard::new(keys, 1);
    let mut corpus = Corpus::default();
    for i in 32..120u16 {
        corpus.bigrams.push((i, i + 1, 100));
        corpus.trigrams.push((i, i + 1, i + 2, 50));
    }

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&keyboard, &corpus, &rubric, &[]);

    let mut layout_keys: Vec<u16> = (32..32 + 30).collect();
    fastrand::shuffle(&mut layout_keys);
    let mut layout = layout_keys.clone();

    let mut pos_map = vec![255u8; 65536];
    for (i, &code) in layout.iter().enumerate() {
        pos_map[code as usize] = i as u8;
    }

    for _ in 0..100 {
        let i = fastrand::usize(0..30);
        let j = fastrand::usize(0..30);
        if i == j {
            continue;
        }

        let score_before = engine.score_raw(&layout);

        let delta = engine.calculate_swap_delta(&layout, &pos_map, i, j);

        let code_i = layout[i] as usize;
        let code_j = layout[j] as usize;
        layout.swap(i, j);
        pos_map[code_i] = j as u8;
        pos_map[code_j] = i as u8;

        let score_after = engine.score_raw(&layout);

        assert_eq!(
            score_after.saturating_sub(score_before),
            delta,
            "Delta mismatch at swap {} <-> {}",
            i,
            j
        );
    }
}
