#[cfg(test)]
mod tests {
    use keyforge_model::{KeyNode, Keyboard, Corpus, Rubric, Layout, KeyCode};
    use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
    use keyforge_physics::ScoringEngine;
    use std::sync::Arc;

    fn setup_szr35_mock() -> (Keyboard, Corpus, Rubric) {
        // Mock a 35-key layout like szr35
        let keys: Vec<_> = (0..35)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{}", i),
                hand: HandIndex((i > 17) as u8),
                finger: FingerIndex((i % 5) as u8),
                row: RowIndex((i / 5) as i8),
                col: ColIndex((i % 5) as i8),
                x: (i % 5) as f32,
                y: (i / 5) as f32,
                ..Default::default()
            })
            .collect();
        let kb = Keyboard::new(keys, 1).unwrap();
        
        let mut corpus = Corpus::default();
        corpus.char_freqs[101] = 1200; // 'e'
        corpus.char_freqs[44] = 150;   // ','
        corpus.char_freqs[32] = 2000;  // ' ' (Space)
        
        // Add some bigrams to make it realistic
        corpus.bigrams.push((101, 32, 500)); // 'e '
        corpus.bigrams.push((32, 101, 400)); // ' e'
        
        let rubric = Rubric::default();
        (kb, corpus, rubric)
    }

    #[test]
    fn test_user_scenario_scoring() {
        let (kb, corpus, rubric) = setup_szr35_mock();
        let engine = ScoringEngine::new(&Arc::new(kb), &Arc::new(corpus), &Arc::new(rubric), &[]).unwrap();

        // 1. Current Layout
        // e (101) at index 0 (expensive) AND index 11 (cheap)
        // , (44) at index 10 (cheap)
        // Space (32) at index 34 (thumb, very good)
        let mut layout_keys = vec![KeyCode(0); 35];
        for i in 0..35 { layout_keys[i] = KeyCode(i as u16 + 200); }
        
        layout_keys[0] = KeyCode(101);  // e (redundant)
        layout_keys[11] = KeyCode(101); // e (home/best)
        layout_keys[10] = KeyCode(44);  // ,
        layout_keys[34] = KeyCode(32);  // Space
        
        let l_current = Layout::new_unchecked(layout_keys.clone());
        let s_current = engine.score(&l_current).unwrap();
        println!("Current Score: {}", s_current);

        // 2. Swapped E(0) and ,(10)
        // Since 'e' already has a best position at 11, moving it from 0 to 10 
        // should have virtually 0 monogram impact.
        let mut l2_keys = layout_keys.clone();
        l2_keys.swap(0, 10);
        let l_e_comma = Layout::new_unchecked(l2_keys);
        let s_e_comma = engine.score(&l_e_comma).unwrap();
        println!("E(0)/, Swap Score: {}", s_e_comma);
        let imp_e_comma = (s_current - s_e_comma) / s_current * 100.0;
        println!("E(0)/, Improvement: {}%", imp_e_comma);

        // 3. Swapped E and Right-hand Space (index 34)
        let mut l3_keys = layout_keys.clone();
        l3_keys.swap(0, 34);
        let l_e_space = Layout::new_unchecked(l3_keys);
        let s_e_space = engine.score(&l_e_space).unwrap();
        println!("E/Space Swap Score: {}", s_e_space);
        let imp_e_space = (s_current - s_e_space) / s_current * 100.0;
        println!("E/Space Improvement: {}%", imp_e_space);

        // Heuristics Check
        let suggestions = engine.suggest_improvements(&l_current);
        println!("Top Suggestions:");
        for sug in suggestions.iter().take(5) {
            println!("  {} <-> {}: {}%", sug.key_a, sug.key_b, sug.improvement_pct);
        }
        
        // Assertions: 
        // 1. E/Space should be better than E/,
        assert!(s_e_space < s_e_comma, "E/Space should score better than E/,");
        
        // 2. E/, improvement should NOT be huge if E/Space is the real winner
        // (This depends on the mock costs, but we want to see it reflected in suggestions)
    }
}
