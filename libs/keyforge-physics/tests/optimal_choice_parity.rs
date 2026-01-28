#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-physics/tests/optimal_choice_parity.rs
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_physics::{EngineCompilationContext, EngineFactory};
    use std::sync::Arc;

    fn mock_cost_model() -> CostModel {
        let json = r#"{
        "version": "2.0",
        "description": "Test",
        "unit": "pts",
        "models": {
            "model_a_row_staggered": {
                "description": "Test Model",
                "static_costs": {
                    "universal_hand": {
                        "thumb": { "pos_1": 100.0, "pos_2": 50.0 },
                        "index": { "base": { "r0": 100.0, "r1": 200.0 } },
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
    fn test_optimal_choice_monogram() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex(0), // cost 100
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex(1), // cost 200
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap(),
        );
        let mut corpus_val = Corpus::default();
        let mut freqs = corpus_val.char_freqs.to_vec();
        freqs[97] = 1000;
        corpus_val.char_freqs = Arc::from(freqs);
        let corpus = Arc::new(corpus_val);
        let cm = Arc::new(mock_cost_model());
        let ctx = keyforge_physics::ScoringContext::new(
            kb,
            corpus,
            Arc::new(Rubric::default()),
            cm,
            keyforge_model::config::EngineConfig::default(),
        );

        let engine = EngineFactory::new_generic(&ctx).unwrap();
        let oracle = EngineFactory::new_exact(&ctx).unwrap();

        // Layout where 'a' is on BOTH keys
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(97)]);

        let score_engine = engine.score(&layout).unwrap();
        let score_oracle = oracle.score(&layout).unwrap();

        assert_eq!(score_engine, score_oracle);
    }

    #[test]
    fn test_optimal_choice_bigram() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                x: 0.0,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                x: 1.0,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                x: 10.0, // Far away
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap(),
        );
        let mut corpus_val = Corpus::default();
        corpus_val.bigrams = vec![(97, 98, 1000)].into();
        let corpus = Arc::new(corpus_val);

        let rubric = Rubric::builder().travel_lat(1.0).build();
        let rubric_arc = Arc::new(rubric);

        let cm = Arc::new(mock_cost_model());
        let ctx = keyforge_physics::ScoringContext::new(
            kb,
            corpus,
            rubric_arc,
            cm,
            keyforge_model::config::EngineConfig::default(),
        );

        let engine = EngineFactory::new_generic(&ctx).unwrap();
        let oracle = EngineFactory::new_exact(&ctx).unwrap();

        // Layout where 'a' is at index 0, and 'b' is at BOTH index 1 and 2
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(98)]);

        let score_engine = engine.score(&layout).unwrap();
        let score_oracle = oracle.score(&layout).unwrap();

        assert_eq!(score_engine, score_oracle);
    }

    #[test]
    fn test_optimal_choice_trigram() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::LEFT,
                finger: FingerIndex::PINKY,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                ..Default::default()
            },
            KeyNode {
                index: 3,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap(),
        );
        let mut corpus_val = Corpus::default();
        corpus_val.trigrams = vec![(97, 98, 99, 1000)].into();
        let corpus = Arc::new(corpus_val);

        let rubric = Rubric::builder().roll_bonus(100.0).redirect(500.0).build();
        let rubric_arc = Arc::new(rubric);

        let cm = Arc::new(mock_cost_model());
        let ctx = keyforge_physics::ScoringContext::new(
            kb,
            corpus,
            rubric_arc,
            cm,
            keyforge_model::config::EngineConfig::default(),
        );

        let engine = EngineFactory::new_generic(&ctx).unwrap();
        let oracle = EngineFactory::new_exact(&ctx).unwrap();

        let layout =
            Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99), KeyCode(99)]);

        let score_engine = engine.score(&layout).unwrap();
        let score_oracle = oracle.score(&layout).unwrap();

        assert_eq!(score_engine, score_oracle);
    }
}
