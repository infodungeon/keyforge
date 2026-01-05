use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::CorpusSource;
use keyforge_model::Layout;
use keyforge_model::types::KeyCode;
use keyforge_physics::ScoringEngine;
use std::path::PathBuf;

#[tokio::test]
async fn debug_space_lifecycle() {
    // 1. Setup Provider
    let root = PathBuf::from("../../sandbox/client");
    let provider = FsProvider::new(root);

    // 2. Load Corpus
    println!("--- CORPUS ANALYSIS ---");
    let sources = vec![CorpusSource {
        id: "text/en_std".to_string(),
        weight: 1.0,
        hash: None,
    }];
    
    let corpus = match provider.load_corpus(&sources).await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to load corpus: {}", e);
            return;
        }
    };
    
    let space_freq = corpus.char_freqs[32];
    let enter_freq = corpus.char_freqs[10];
    let bksp_freq = corpus.char_freqs[8];
    let total_chars: u32 = corpus.char_freqs.iter().sum();

    println!("Total Chars: {}", total_chars);
    println!("Space (32): {} ({:.2}%)", space_freq, (space_freq as f32 / total_chars as f32) * 100.0);
    println!("Enter (10): {} ({:.2}%)", enter_freq, (enter_freq as f32 / total_chars as f32) * 100.0);
    println!("Bksp (8):   {} ({:.2}%)", bksp_freq, (bksp_freq as f32 / total_chars as f32) * 100.0);

    // 3. Load Registry
    println!("\n--- REGISTRY ANALYSIS ---");
    let registry = match provider.load_keycodes("keycodes").await {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to load keycodes: {}", e);
            return;
        }
    };
    
    let kc_spc = registry.get_code("KC_SPC");
    let kc_ent = registry.get_code("KC_ENT");
    let kc_bspc = registry.get_code("KC_BSPC");
    let space_alias = registry.get_code("SPACE");

    println!("KC_SPC: {:?}", kc_spc);
    println!("SPACE:  {:?}", space_alias);
    println!("KC_ENT: {:?}", kc_ent);
    println!("KC_BSPC: {:?}", kc_bspc);

    // 4. Physics Analysis
    println!("\n--- PHYSICS SCALING ANALYSIS ---");
    // Load a dummy keyboard (ortho_30 is small and simple)
    let kb_def = match provider.load_keyboard("ortho_30").await {
        Ok(k) => k,
        Err(e) => {
            println!("Failed to load keyboard: {}", e);
            return;
        }
    };
    let kb = keyforge_model::Keyboard::new(kb_def.geometry.keys.clone(), kb_def.geometry.home_row).unwrap();
    
    let rubric = keyforge_model::Rubric::default();
    
    // Create a layout with Space at index 0
    let mut keys = vec![KeyCode(0); kb.count()];
    if let Some(code) = kc_spc {
        keys[0] = code;
    }
    let layout = Layout::new_unchecked(keys);
    
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).expect("Failed to build engine");
    let report = engine.analyze(&layout).expect("Failed to analyze");
    
    println!("Space Key Index: 0");
    println!("Penalty Map [0]: {}", report.penalty_map[0]);
    println!("Total Score: {}", report.score);
    
    // Check if penalty map is normalized
    if report.penalty_map[0] < 1.0 && space_freq > 1000 {
        println!("⚠️  WARNING: Penalty Map appears to be normalized (Value < 1.0)");
    } else {
        println!("✅ Penalty Map appears to be raw (Value >= 1.0)");
    }
}
