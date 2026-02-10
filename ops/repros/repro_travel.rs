// ops/repros/repro_travel.rs

use keyforge_model::{Corpus, Keyboard, Rubric, layout::Layout};
use keyforge_model::geometry::KeyboardDefinition;

fn main() {
    // SAFETY: ARCH-005 Exception: Reproduction script requires loading system keyboard models
    // for analysis. This falls outside the pure physics/evolution kernel boundary.
    let path = std::path::Path::new("data/system/keyboards/models/szr35.mpk.zst");
    // SAFETY: TYPE-003 Exception: Reproduction script.
    let file = std::fs::File::open(path).expect("Failed to open file");
    let decoder = zstd::Decoder::new(file).expect("Failed to create decoder");
    let kb_def: KeyboardDefinition = rmp_serde::from_read(decoder).expect("Failed to deserialize");
    
    let kb = Keyboard::new(kb_def.geometry.keys().clone(), kb_def.geometry.home_row(), "test".into()).expect("Failed to create keyboard");

    // Simple corpus: 
    // 'q' (idx 0), 'a' (idx 5), Space (idx 16/34)
    let mut corpus = Corpus::default();
    let q = 113u16;
    let a = 97u16;
    let space = 32u16;

    corpus.char_freqs[usize::from(q)] = 1000;
    corpus.char_freqs[usize::from(a)] = 1000;
    corpus.char_freqs[usize::from(space)] = 1000;
    
    // Q -> A transition (SFB: Pinky)
    corpus.bigrams.push((q, a, 500)); 
    // A -> Space transition (Different finger)
    corpus.bigrams.push((a, space, 500));

    let rubric = Rubric::default();

    // Layout: SZR35 (CoDH style-ish)
    let mut layout_codes = vec![keyforge_model::types::KeyCode::new(0); 36];
    layout_codes[0] = keyforge_model::types::KeyCode::new(q);
    layout_codes[5] = keyforge_model::types::KeyCode::new(a);
    layout_codes[16] = keyforge_model::types::KeyCode::new(32); // SpaceL
    layout_codes[34] = keyforge_model::types::KeyCode::new(32); // SpaceR
    
    let layout = Layout::new_unchecked(layout_codes);

    let engine = keyforge_physics::ScoringEngine::new(&kb, &corpus, &rubric, &[]).expect("Failed to create engine");

    // 1. Both (Bilateral)
    let report_both = engine.analyze(&layout).expect("Failed to analyze bilateral");
    
    // 2. Left Only
    let mut layout_left_codes = layout.keys().clone();
    layout_left_codes[34] = keyforge_model::types::KeyCode::new(0); // Mask right space
    let layout_left = Layout::new_unchecked(layout_left_codes);
    let report_left = engine.analyze(&layout_left).expect("Failed to analyze left only");

    // 3. Right Only
    let mut layout_right_codes = layout.keys().clone();
    layout_right_codes[16] = keyforge_model::types::KeyCode::new(0); // Mask left space
    let layout_right = Layout::new_unchecked(layout_right_codes);
    let report_right = engine.analyze(&layout_right).expect("Failed to analyze right only");
    
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
