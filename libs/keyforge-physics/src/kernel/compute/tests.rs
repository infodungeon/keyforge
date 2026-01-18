#[cfg(test)]
mod tests {
    use crate::kernel::compute::{PosMap, calculate_swap_delta};
    use crate::kernel::types::ValidatedLayout;
    use crate::ScoringEngine;
    use keyforge_model::{
        Corpus, KeyNode, Keyboard, Layout, Rubric, CostModel,
        types::{HandIndex, FingerIndex, KeyCode}
    };

    fn setup_kb_robust() -> Keyboard {
        let keys: Vec<KeyNode> = (0..5).map(|i| KeyNode {
            index: i,
            hand: HandIndex(0),
            finger: FingerIndex(i as u8),
            x: i as f32,
            ..Default::default()
        }).collect();
        Keyboard::new(keys, 0).unwrap()
    }

    fn mock_cost_model() -> CostModel {
        let json = r#"{
            "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
            "models": {
                "model_a_row_staggered": {
                    "description": "Test Model",
                    "static_costs": {
                        "universal_hand": {
                            "thumb": { "pos_1": 100.0 },
                            "index": { "base": { "r0": 100.0 } },
                            "middle": { "base": { "r0": 100.0 } },
                            "ring": { "base": { "r0": 100.0 } },
                            "pinky": { "base": { "r0": 100.0 } }
                        }
                    }
                }
            },
            "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_math_boundaries_infinity() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 1000));

        let rubric = Rubric {
            travel_lat: f32::INFINITY,
            ..Rubric::default()
        };

        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();

        assert!(score > 1_000_000.0);
        assert!(score.is_finite());
    }

    #[test]
    fn test_math_boundaries_nan() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 1000));

        let rubric = Rubric {
            travel_lat: f32::NAN,
            ..Rubric::default()
        };

        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();

        assert!(score >= 0.0);
        assert!(!score.is_nan());
    }

    #[test]
    fn test_saturation_protection() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, u32::MAX));

        let rubric = Rubric {
            travel_lat: 1_000_000.0,
            ..Rubric::default()
        };

        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();
        assert!(score.is_finite());
    }

    #[test]
    fn test_missing_keys_in_layout() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(0), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 100)); 

        let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();
        let score = engine.score(&layout).unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_swap_delta_bounds() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(100), KeyCode(101)]);
        let mut corpus = Corpus::default();
        corpus.bigrams.push((97, 98, 100));

        let engine = ScoringEngine::new(&kb, &corpus, &Rubric::default(), &mock_cost_model()).unwrap();
        let mut pos_map_data = vec![65535u16; 65536];
        for (i, &code) in layout.keys.iter().enumerate() {
            pos_map_data[code.0 as usize] = i as u16;
        }

        let validated = ValidatedLayout::new(&layout.keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let pm = PosMap::from_scratch(
            &layout.keys, 
            engine.key_count(), 
            &mut starts, 
            &mut counts, 
            &mut indices, 
            &engine.context().sorted_unique_keys,
            &engine.context().key_rank_map
        );

        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 100);
        assert_eq!(delta, 0);
    }

    #[test]
    fn test_analyze_layout_empty() {
        let kb = setup_kb_robust();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        let layout = Layout::new_unchecked(vec![]);
        let validated_res = ValidatedLayout::new(&layout.keys, engine.key_count());
        assert!(validated_res.is_err());
    }

    #[test]
    fn test_delta_internals_manual() {
        let keys = vec![
            KeyNode { index: 0, x: 0.0, ..Default::default() },
            KeyNode { index: 1, x: 10.0, ..Default::default() },
            KeyNode { index: 2, x: 20.0, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 1, 100));
        corpus.trigrams.push((0, 1, 2, 100));
        
        let mut rubric = Rubric::default();
        rubric.travel_lat = 1.0;
        
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        let layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
        let mut pos_map_data = vec![65535u16; 65536];
        pos_map_data[0] = 0; pos_map_data[1] = 1; pos_map_data[2] = 2;
        
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let pm = PosMap::from_scratch(
            &layout_keys, 
            engine.key_count(), 
            &mut starts, 
            &mut counts, 
            &mut indices, 
            &engine.context().sorted_unique_keys,
            &engine.context().key_rank_map
        );

        let score_before = engine.score_raw(&layout_keys).unwrap();
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 1, 2);
        
        let mut swapped_keys = layout_keys.clone();
        swapped_keys.swap(1, 2);
        let score_after = engine.score_raw(&swapped_keys).unwrap();
        
        assert_eq!(score_after - score_before, delta, "Manual delta check failed");
    }

    #[test]
    fn test_delta_self_loop() {
        let keys = vec![
            KeyNode { index: 0, x: 0.0, ..Default::default() },
            KeyNode { index: 1, x: 10.0, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        
        let mut corpus = Corpus::default();
        corpus.bigrams.push((0, 0, 100));
        
        let mut rubric = Rubric::default();
        rubric.travel_lat = 1.0;
        rubric.trigram_limit = 0; 
        
        let engine = ScoringEngine::new(&kb, &corpus, &rubric, &mock_cost_model()).unwrap();
        
        let layout_keys = vec![KeyCode(0), KeyCode(1)];
        let mut pos_map_data = vec![65535u16; 65536];
        pos_map_data[0] = 0; pos_map_data[1] = 1;
        
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [0u16; 512];
        let pm = PosMap::from_scratch(
            &layout_keys, 
            engine.key_count(), 
            &mut starts, 
            &mut counts, 
            &mut indices, 
            &engine.context().sorted_unique_keys,
            &engine.context().key_rank_map
        );

        let score_before = engine.score_raw(&layout_keys).unwrap();
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 1);
        
        let mut swapped_keys = layout_keys.clone();
        swapped_keys.swap(0, 1);
        let score_after = engine.score_raw(&swapped_keys).unwrap();
        
        assert_eq!(score_after - score_before, delta, "Self loop delta check failed");
    }
}