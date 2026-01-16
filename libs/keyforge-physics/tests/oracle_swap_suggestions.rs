
// libs/keyforge-physics/tests/debug_smart_assist.rs

#[cfg(test)]
mod tests {
    use keyforge_infra::FsProvider;
    use keyforge_core::loader::AssetLoader;
    use keyforge_physics::ScoringEngine;
    use keyforge_model::{Rubric, Layout, KeyCode, Keyboard, CorpusSource};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn repro_bad_swaps() {
        let root = PathBuf::from("/home/robert/Documents/KeyboardLayouts/DataDrivenAnalysis/keyforge/data");
        let loader = FsProvider::new(root);
        
        let kb_def = loader.load_keyboard("szr35").await
            .expect("Failed to load szr35");
            
        let kb = Keyboard::new(kb_def.geometry.keys.clone(), kb_def.geometry.home_row)
            .expect("Failed to create keyboard");

        // Load actual en_std corpus for realistic bigram/trigram data
        let sources = vec![CorpusSource {
            id: "text/en_std".to_string(),
            weight: 1.0,
            hash: None,
        }];
        let corpus = loader.load_corpus(&sources).await
            .expect("Failed to load en_std corpus");
        
        println!("Corpus loaded: {} chars, {} bigrams, {} trigrams", 
            corpus.char_freqs.iter().filter(|&&f| f > 0).count(),
            corpus.bigrams.len(),
            corpus.trigrams.len());

        let rubric = Rubric::default();
        
        // Load default cost matrix to reproduce the issue
        let cost_data = loader.load_cost_matrix("cost_matrix").await
            .expect("Failed to load cost matrix");
        let overrides = cost_data.resolve(&kb_def.geometry);

        println!("Loaded cost matrix with {} overrides", overrides.len());
        
        // Debug: Show overrides related to our problem indices (25, 30, 31, 32)
        let problem_indices = [25, 30, 31, 32];
        println!("\n=== Cost Overrides for Problem Indices ===");
        for (from_idx, to_idx, cost) in &overrides {
            if problem_indices.contains(from_idx) || problem_indices.contains(to_idx) {
                let from_label = &kb_def.geometry.keys[*from_idx].label;
                let to_label = &kb_def.geometry.keys[*to_idx].label;
                println!("  {} ({}) -> {} ({}): cost = {:.2}", 
                    from_label, from_idx, to_label, to_idx, cost);
            }
        }

        // Create Scoring Engine
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &overrides)
            .expect("Failed to create scoring engine");

        println!("=== SZR35 Key Map (Detailed) ===");
        let ctx = engine.context();
        for i in 0..kb.count() {
            let k = &kb.keys[i];
            // Format finger: 0=LP, 1=LR, 2=LM, 3=LI, 4=LT, 5=RT, 6=RI, 7=RM, 8=RR, 9=RP
            println!("Idx {:2}: {:<6} F:{:?} ({:?}) Hnd:{:?} Home:{} X:{:.2} Y:{:.2}", 
                i, k.label, k.finger, ctx.fingers[i], k.hand, k.is_home, k.x, k.y);
        }
        
        println!("\nRubric Finger Effort: {:?}", rubric.finger_effort);

        // Use Colemak-DH layout from keyboard definition
        let layout_str = kb_def.layouts.get("Colemak-DH")
            .expect("Colemak-DH layout not found in keyboard definition");
        
        println!("\n=== Parsing Colemak-DH layout from keyboard definition ===");
        println!("Layout string: {}", layout_str);
        
        // Parse layout string using keyforge_adapter
        use keyforge_adapter::conversion::parse_layout_string_permissive;
        use keyforge_model::keycodes::KeycodeRegistry;
        
        // Load keycode registry for proper parsing
        let registry = loader.load_keycodes("keycodes").await
            .expect("Failed to load keycode registry");
        let layout = parse_layout_string_permissive(layout_str, kb.count(), &registry);
        
        // Debug: Print some key mappings
        println!("\n=== Parsed Layout Keys ===");
        for (i, key_code) in layout.keys.iter().take(30).enumerate() {
            if key_code.0 > 0 {
                let ch = if key_code.0 < 128 { 
                    format!("{}", key_code.0 as u8 as char)
                } else {
                    format!("0x{:04X}", key_code.0)
                };
                println!("Idx {}: {} (code {})", i, ch, key_code.0);
            }
        }

        println!("\n=== Initial Analysis (SZR35 Colemak-DH) ===");
        let report = engine.analyze(&layout).unwrap();
        println!("Total Score: {:.2}", report.score);
        
        // Print key checks
        println!("Idx 25 (E): {}", if layout.keys[25] == KeyCode(b'E' as u16) { "OK" } else { "FAIL" });
        println!("Idx 30 (Comm): {}", if layout.keys[30] == KeyCode(b',' as u16) { "OK" } else { "FAIL" });
        println!("Idx 31 (Dot): {}", if layout.keys[31] == KeyCode(b'.' as u16) { "OK" } else { "FAIL" });
        
        // Check suggestions
        
        println!("\n=== Suggestions (Include Thumbs: TRUE) ===");
        let suggestions_thumbs = engine.suggest_improvements(&layout, true);
        for (i, s) in suggestions_thumbs.iter().take(5).enumerate() {
            println!("{}. Swap {} ({}) <-> {} ({}): Improvement {:.2}%", 
                i+1, s.key_a, s.index_a, s.key_b, s.index_b, s.improvement_pct);
        }
        
        // ORACLE ASSERTION: With thumbs enabled, top suggestion should be moving 'e' to a thumb position
        assert!(!suggestions_thumbs.is_empty(), "Should have suggestions with thumbs enabled");
        let top_thumb = &suggestions_thumbs[0];
        let thumb_indices = [16, 17, 33, 34, 35];
        let e_code = "101"; // ASCII code for 'e'
        assert!(
            (top_thumb.key_a == e_code && thumb_indices.contains(&top_thumb.index_b)) ||
            (top_thumb.key_b == e_code && thumb_indices.contains(&top_thumb.index_a)),
            "Top suggestion with thumbs should move 'e' (code 101) to a thumb position, got: {} ({}) <-> {} ({})",
            top_thumb.key_a, top_thumb.index_a, top_thumb.key_b, top_thumb.index_b
        );
        println!("✓ PASSED: Top suggestion correctly moves 'e' to thumb");

        println!("\n=== Suggestions (Include Thumbs: FALSE) ===");
        let suggestions_no_thumbs = engine.suggest_improvements(&layout, false);
        for (i, s) in suggestions_no_thumbs.iter().take(5).enumerate() {
            println!("{}. Swap {} ({}) <-> {} ({}): Improvement {:.2}%",
                i+1, s.key_a, s.index_a, s.key_b, s.index_b, s.improvement_pct);
        }
        
        // ORACLE ASSERTION: With thumbs disabled, should NOT suggest moving 'e' to bottom-row punctuation
        let punctuation_indices = [30, 31, 32]; // , . /
        let bad_e_swaps = suggestions_no_thumbs.iter().filter(|s| {
            (s.key_a == e_code && punctuation_indices.contains(&s.index_b)) ||
            (s.key_b == e_code && punctuation_indices.contains(&s.index_a))
        }).count();
        
        
        // DEBUG: Analyze the top bad suggestion if it exists
        if let Some(bad_swap) = suggestions_no_thumbs.iter().find(|s| 
            (s.key_a == e_code && punctuation_indices.contains(&s.index_b)) ||
            (s.key_b == e_code && punctuation_indices.contains(&s.index_a))
        ) {
            println!("\n=== ANALYZING BAD SWAP: {} ({}) <-> {} ({}) ===", 
                bad_swap.key_a, bad_swap.index_a, bad_swap.key_b, bad_swap.index_b);
            println!("Reported improvement: {:.2}%", bad_swap.improvement_pct);
            
            // Get the actual indices
            let idx_e = if bad_swap.key_a == e_code { bad_swap.index_a } else { bad_swap.index_b };
            let idx_punct = if bad_swap.key_a == e_code { bad_swap.index_b } else { bad_swap.index_a };
            
            println!("\nIndex {} (e): {} - Finger {:?}, Home: {}", 
                idx_e, kb.keys[idx_e as usize].label, kb.keys[idx_e as usize].finger, kb.keys[idx_e as usize].is_home);
            println!("Index {} (punct): {} - Finger {:?}, Home: {}", 
                idx_punct, kb.keys[idx_punct as usize].label, kb.keys[idx_punct as usize].finger, kb.keys[idx_punct as usize].is_home);
            
            // Test with NO cost matrix to see if suggestion persists
            println!("\n=== Testing WITHOUT cost matrix overrides ===");
            let engine_no_cost = ScoringEngine::new(&kb, &corpus, &rubric, &[])
                .expect("Failed to create no-cost engine");
            let suggestions_no_cost = engine_no_cost.suggest_improvements(&layout, false);
            
            let bad_swaps_no_cost = suggestions_no_cost.iter().filter(|s| {
                (s.key_a == e_code && punctuation_indices.contains(&s.index_b)) ||
                (s.key_b == e_code && punctuation_indices.contains(&s.index_a))
            }).count();
            
            println!("Bad swaps WITHOUT cost matrix: {}", bad_swaps_no_cost);
            if bad_swaps_no_cost > 0 {
                println!("Top 3 suggestions WITHOUT cost matrix:");
                for (i, s) in suggestions_no_cost.iter().take(3).enumerate() {
                    println!("  {}. {} ({}) <-> {} ({}): {:.2}%", 
                        i+1, s.key_a, s.index_a, s.key_b, s.index_b, s.improvement_pct);
                }
            }
        }

        
        assert_eq!(bad_e_swaps, 0, 
            "Should NOT suggest moving 'e' to punctuation (indices 30,31,32) when thumbs disabled, but found {} such suggestions",
            bad_e_swaps
        );
        println!("✓ PASSED: Does not suggest moving 'e' to bottom-row punctuation");
        
        println!("\n=== ORACLE TEST PASSED ===");
        println!("Expected behavior:");
        println!("  1. With thumbs enabled: Suggests moving 'e' to thumb position");
        println!("  2. With thumbs disabled: Does NOT suggest moving 'e' to punctuation");
    }
}
