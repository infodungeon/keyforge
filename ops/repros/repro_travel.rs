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
    
    let kb = Keyboard::new(kb_def.geometry.keys.clone(), kb_def.geometry.home_row).unwrap();

    // Simple corpus with lots of spaces
    let mut corpus = Corpus::default();
    let space_code = 32u16; // ASCII space
    corpus.char_freqs[space_code as usize] = 1000;
    corpus.char_freqs[113] = 1000; // 'q' 
    
    // 'q' (Key 0) to Space transition
    corpus.bigrams.push((113, space_code, 500)); 

    let rubric = Rubric::default();

    // Layout: 
    // Key 0 is 'q' (Top Row, Pinky Hand 0)
    // Key 16 is SpaceL (Thumb Hand 0)
    // Key 34 is SpaceR (Thumb Hand 1)
    let mut layout_codes = vec![keyforge_model::types::KeyCode(0); 36];
    layout_codes[0] = keyforge_model::types::KeyCode(113); // 'q'
    layout_codes[16] = keyforge_model::types::KeyCode(32); // SpaceL
    layout_codes[34] = keyforge_model::types::KeyCode(32); // SpaceR
    
    let layout = Layout::new_unchecked(layout_codes);

    let engine = keyforge_physics::ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

    // 1. Both (Bilateral)
    let report_both = engine.analyze(&layout).unwrap();
    println!("Bilateral Travel: {:.4}", report_both.distance);

    // 2. Left Only
    let mut layout_left_codes = layout.keys.clone();
    layout_left_codes[34] = keyforge_model::types::KeyCode(0); // Mask right space
    let layout_left = Layout::new_unchecked(layout_left_codes);
    let report_left = engine.analyze(&layout_left).unwrap();
    println!("Left Only Travel: {:.4}", report_left.distance);

    // 3. Right Only
    let mut layout_right_codes = layout.keys.clone();
    layout_right_codes[16] = keyforge_model::types::KeyCode(0); // Mask left space
    let layout_right = Layout::new_unchecked(layout_right_codes);
    let report_right = engine.analyze(&layout_right).unwrap();
    println!("Right Only Travel: {:.4}", report_right.distance);
    
    if report_both.distance > report_left.distance + 0.0001 || report_both.distance > report_right.distance + 0.0001 {
        println!("❌ LOGICAL INCONSISTENCY DETECTED: Bilateral travel is worse than single-hand!");
        println!("  Both:  {:.4}", report_both.distance);
        println!("  Left:  {:.4}", report_left.distance);
        println!("  Right: {:.4}", report_right.distance);
    } else {
        println!("✅ Bilateral travel is optimal.");
    }
}
