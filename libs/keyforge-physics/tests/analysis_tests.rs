use keyforge_physics::ScoringEngine;
use keyforge_model::{Keyboard, KeyNode, Corpus, Rubric};

#[test]
fn test_analyze_layout_comprehensive() {
    let keys: Vec<_> = (0..30).map(|i| KeyNode {
        id: i, label: format!("k{}", i), hand: (i % 2) as u8, finger: (i % 5) as u8,
        row: (i / 10) as i8, col: (i % 10) as i8, x: (i % 10) as f32, y: (i / 10) as f32, is_home: false,
    }).collect();
    let kb = Keyboard::new(keys, 1);
    
    let mut corpus = Corpus::default();
    // SFB: 0 and 2 are same hand (0) and finger (0)
    corpus.bigrams.push((0, 2, 100)); 
    // SFB: 0 and 20 (hand0, finger0, r0 vs r2)
    corpus.bigrams.push((0, 20, 100));
    
    let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &[]);
    let layout = keyforge_model::Layout::new((0..30).collect());
    let report = engine.analyze(&layout);
    
    assert!(report.score >= 0.0);
    assert!(report.sfb_total > 0.0);
}
