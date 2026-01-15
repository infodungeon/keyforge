use keyforge_model::{Corpus, Keyboard, Rubric};
use keyforge_physics::ScoringEngine;
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb_path = "data/system/keyboards/models/szr35.mpk.zst";
    let kb_file = File::open(kb_path)?;
    let kb: Keyboard = rmp_serde::from_read(zstd::Decoder::new(kb_file)?)?;

    println!("Keyboard: SZR35 (Home Row: {})", kb.home_row);

    let corpus = Corpus::default();
    let rubric = Rubric::default();
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[])?;
    let ctx = engine.context();

    println!("\nKey Distances from Home:");
    for (i, k) in kb.keys.iter().enumerate() {
        let dist = ctx.key_home_distances[i];
        if dist < 0.001 || k.is_home {
            println!("  [HOME] Idx {}: {} (Finger {:?}, Hand {:?}) dist: {:.4}", 
                i, k.label, k.finger, k.hand, dist);
        } else if dist < 0.1 {
             println!("  [NEAR] Idx {}: {} (Finger {:?}, Hand {:?}) dist: {:.4}", 
                i, k.label, k.finger, k.hand, dist);
        }
    }

    Ok(())
}
