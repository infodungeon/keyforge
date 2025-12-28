use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use keyforge_physics::ScoringEngine;
use std::sync::Arc;

fn setup_physics_engine() -> ScoringEngine {
    let keys = vec![
        KeyNode { id: 0, label: "A".to_string(), hand: 0, finger: 1, row: 1, col: 0, x: 0.0, y: 1.0, is_home: true },
        KeyNode { id: 1, label: "B".to_string(), hand: 0, finger: 2, row: 1, col: 1, x: 1.0, y: 1.0, is_home: true },
        KeyNode { id: 2, label: "C".to_string(), hand: 0, finger: 3, row: 1, col: 2, x: 2.0, y: 1.0, is_home: true },
        KeyNode { id: 3, label: "D".to_string(), hand: 1, finger: 1, row: 1, col: 6, x: 6.0, y: 1.0, is_home: true },
        // A scissor case: (0, 4) - finger 1 vs finger 1 is SFB, but let's do finger 1 vs finger 2 with row diff
        KeyNode { id: 4, label: "E".to_string(), hand: 0, finger: 1, row: 3, col: 0, x: 0.0, y: 3.0, is_home: false },
    ];
    let keyboard = Arc::new(Keyboard::new(keys, 1));
    
    let mut corpus = Corpus::default();
    corpus.char_freqs[0] = 100; // 'A'
    corpus.char_freqs[1] = 50;  // 'B'
    
    // SFB: A and E are on finger 1, hand 0
    corpus.bigrams.push((0, 4, 10)); 
    
    // Scissor: A (r1, f1) and B (r1, f2) - no. 
    // Let's do B (r1, f2) and E (r3, f1). Hand 0, Fingers 2 and 1, Row diff 2.
    corpus.bigrams.push((1, 4, 5));
    
    // Trigram Roll: C(f=3) -> B(f=2) -> A(f=1) (Outward roll on left hand)
    corpus.trigrams.push((2, 1, 0, 20));
    
    // Trigram Redirect: A -> C -> B 
    corpus.trigrams.push((0, 2, 1, 15));

    let rubric = Arc::new(Rubric::default());
    ScoringEngine::new(&keyboard, &Arc::new(corpus), &rubric, &[])
}

#[test]
fn test_analyze_layout_comprehensive() {
    let engine = setup_physics_engine();
    let layout = Layout::new(vec![0, 1, 2, 3, 4]);
    
    let report = engine.analyze(&layout);
    
    // Monogram Heatmap
    assert_eq!(report.heatmap[0], 100.0);
    assert_eq!(report.heatmap[1], 50.0);
    
    // SFB Ratio
    // Total bigrams = 10 (SFB) + 5 (Scissor) = 15
    // SFB total = 10
    // Ratio = 10/15 = 0.666
    assert!(report.sfb_ratio > 0.6 && report.sfb_ratio < 0.7);
    
    // Scissor
    assert!(report.scissors > 0.0);
    
    // Trigram Stats
    assert!(report.rolls > 0.0);
    assert!(report.redirects > 0.0);
    
    // Hand Balance
    // Left load: 100 (A) + 50 (B) + 0 (C) + 0 (E) = 150
    // Right load: 0 (D) = 0
    // Total: 150
    // Left ratio = 150/150 = 1.0
    // Balance = (1.0 - 0.5) * -2.0 = -1.0
    assert_eq!(report.hand_balance, -1.0);
}

#[test]
fn test_analyze_layout_empty() {
    let engine = setup_physics_engine();
    let layout = Layout::new(vec![]);
    let report = engine.analyze(&layout);
    assert_eq!(report.score, 0.0);
}
