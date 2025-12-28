use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::ScoringEngine;

fn setup_kb(size: usize) -> Keyboard {
    let keys: Vec<KeyNode> = (0..size)
        .map(|i| KeyNode {
            id: i,
            label: format!("k{}", i),
            hand: 0,
            finger: i as u8,
            row: 0,
            col: i as i8,
            x: i as f32,
            y: 0.0,
            is_home: true,
        })
        .collect();
    Keyboard::new(keys, 0)
}

#[test]
fn test_swap_delta_bounds_strict() {
    let kb = setup_kb(5);
    let layout_vec: Vec<u16> = (0..10).collect();
    let layout = Layout::new(layout_vec);

    let corpus = Corpus::default();
    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);

    let mut pos_map = vec![65535u16; 65536];
    for i in 0..5 {
        pos_map[i] = i as u16;
    }

    let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 8);
    assert_eq!(delta, 0);

    let delta = engine.calculate_swap_delta(&layout.keys, &pos_map, 8, 0);
    assert_eq!(delta, 0);
}

#[test]
fn test_swap_delta_reflexive_skips() {
    let kb = setup_kb(5);
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 0, 100));
    corpus.trigrams.push((0, 0, 0, 100));
    corpus.trigrams.push((0, 1, 0, 100));

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);

    let mut pos_map = vec![65535u16; 65536];
    for i in 0..5 {
        pos_map[i as usize] = i as u16;
    }

    let _ = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 1);
}

#[test]
fn test_swap_delta_math_coverage() {
    let kb = setup_kb(5);
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);

    let mut corpus = Corpus::default();
    corpus.bigrams.push((0, 1, 100));
    corpus.bigrams.push((1, 0, 100));
    corpus.trigrams.push((0, 1, 2, 100));
    corpus.trigrams.push((1, 0, 2, 100));
    corpus.trigrams.push((1, 2, 0, 100));

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]);

    let mut pos_map = vec![65535u16; 65536];
    for i in 0..5 {
        pos_map[i as usize] = i as u16;
    }

    let _ = engine.calculate_swap_delta(&layout.keys, &pos_map, 0, 3);
}
