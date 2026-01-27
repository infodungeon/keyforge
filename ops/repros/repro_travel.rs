#![allow(clippy::unwrap_used, clippy::expect_used)]
use keyforge_model::{Corpus, Keyboard, Rubric, layout::Layout};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_physics::ScoringEngine;
use std::fs::File;
use std::path::Path;

fn main() {
    let path = Path::new("data/system/keyboards/models/szr35.mpk.zst");
    let file = File::open(path).expect("Failed to open file");
    let decoder = zstd::Decoder::new(file).expect("Failed to create decoder");
    let kb_def: KeyboardDefinition = rmp_serde::from_read(decoder).expect("Failed to deserialize");
    
    let kb = Keyboard::new(kb_def.geometry.keys.clone(), kb_def.geometry.home_row, "test".into()).unwrap();

    // Simple corpus: 
    // 'q' (idx 0), 'a' (idx 5), Space (idx 16/34)
    let mut corpus = Corpus::default();
    let q = 113u16;
    let a = 97u16;
    let space = 32u16;

    corpus.char_freqs[q as usize] = 1000;
    corpus.char_freqs[a as usize] = 1000;
    corpus.char_freqs[space as usize] = 1000;
    
    // Q -> A transition (SFB: Pinky)
    corpus.bigrams.push((q, a, 500)); 
    // A -> Space transition (Different finger)
    corpus.bigrams.push((a, space, 500));

    let rubric = Rubric::default();

    // Layout: SZR35 (CoDH style-ish)
    let mut layout_codes = vec![keyforge_model::types::KeyCode(0); 36];
    layout_codes[0] = keyforge_model::types::KeyCode(q);
    layout_codes[5] = keyforge_model::types::KeyCode(a);
    layout_codes[16] = keyforge_model::types::KeyCode(32); // SpaceL
    layout_codes[34] = keyforge_model::types::KeyCode(32); // SpaceR
    
    let layout = Layout::new_unchecked(layout_codes);

    let engine = keyforge_physics::ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

    // 1. Both (Bilateral)
    let report_both = engine.analyze(&layout).unwrap();
    
    // 2. Left Only
    let mut layout_left_codes = layout.keys.clone();
    layout_left_codes[34] = keyforge_model::types::KeyCode(0); // Mask right space
    let layout_left = Layout::new_unchecked(layout_left_codes);
    let report_left = engine.analyze(&layout_left).unwrap();

    // 3. Right Only
    let mut layout_right_codes = layout.keys.clone();
    layout_right_codes[16] = keyforge_model::types::KeyCode(0); // Mask left space
    let layout_right = Layout::new_unchecked(layout_right_codes);
    let report_right = engine.analyze(&layout_right).unwrap();
    
    println!("--- Analytical Results ---");
    println!("Bilateral Distance: {:.4}", report_both.distance);
    println!("Left Only Distance: {:.4}", report_left.distance);
    println!("Right Only Distance: {:.4}", report_right.distance);
    
    if report_both.distance > report_left.distance + 0.0001 && report_both.distance > report_right.distance + 0.0001 {
        println!("❌ LOGICAL INCONSISTENCY DETECTED!");
    } else {
        println!("✅ Consistency Check Passed.");
    }
}
