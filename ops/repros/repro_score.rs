// Reproduction: Score 0 Investigation
// Usage: rustc ops/repros/repro_score.rs && ./repro_score

use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::Corpus;
use keyforge_core::ScoringEngine;
use keyforge_model::Rubric;
use keyforge_model::Layout;
use keyforge_model::KeyCode;
use std::path::PathBuf;
use std::fs::File;

fn main() {
    println!("🔍 Investigating Score 0...");

    let root = PathBuf::from("sandbox/client");
    
    // 1. Load Keyboard
    let kb_path = root.join("system/keyboards/models/ortho_30.mpk.zst"); // Assuming default
    // Fallback to what might be there if ortho_30 isn't
    let kb_path = if kb_path.exists() { kb_path } else {
        root.join("system/keyboards/models/corne.mpk.zst")
    };
    
    println!("   Loading Keyboard: {:?}", kb_path);
    let kb_file = File::open(&kb_path).expect("Failed to open keyboard");
    let kb_def: KeyboardDefinition = rmp_serde::from_read(zstd::Decoder::new(kb_file).unwrap()).expect("Failed to parse keyboard");
    let keyboard = keyforge_model::Keyboard::new(kb_def.geometry.keys, kb_def.geometry.home_row).unwrap();

    // 2. Load Corpus (1grams only for simple check)
    let cp_path = root.join("system/corpora/text/en_std/1grams.mpk.zst");
    println!("   Loading Corpus: {:?}", cp_path);
    let cp_file = File::open(&cp_path).expect("Failed to open corpus");
    let grams: Vec<serde_json::Value> = rmp_serde::from_read(zstd::Decoder::new(cp_file).unwrap()).expect("Failed to parse corpus");
    
    let mut corpus = Corpus::default();
    // Quick hack to populate
    for g in grams {
        if let Some(c_str) = g["char"].as_str() {
             // Hex decode
             if let Ok(c_u8) = u8::from_str_radix(c_str, 16) {
                 let freq = g["freq"].as_u64().unwrap_or(0);
                 corpus.char_freqs[c_u8 as usize] = freq;
             }
        }
    }
    
    let total_freq: u64 = corpus.char_freqs.iter().sum();
    println!("   Corpus Total Freq: {}", total_freq);
    if total_freq == 0 {
        println!("❌ FAIL: Corpus is empty!");
        return;
    }

    // 3. Load Weights
    let wt_path = root.join("system/weights/cost_matrix.mpk.zst");
    println!("   Loading Weights: {:?}", wt_path);
    // We skip loading full weights for this repro, just checking if file exists/is valid
    if !wt_path.exists() {
        println!("❌ FAIL: Weights missing!");
        return;
    }

    // 4. Score
    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&keyboard, &corpus, &rubric, &[]).expect("Failed to build engine");
    
    // Dummy layout
    let layout = Layout::new_unchecked((0..keyboard.keys.len() as u16).map(KeyCode).collect());
    
    let score = engine.score(&layout).unwrap();
    println!("   Calculated Score: {}", score);

    if score > 0.0 {
        println!("✅ PASS: Engine produces non-zero score.");
    } else {
        println!("❌ FAIL: Score is 0.0");
    }
}
