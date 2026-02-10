#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-physics/tests/optimal_choice_parity.rs
<<<<<<< HEAD
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, RowIndex, SpatialUnit};
=======
    use keyforge_model::types::{FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex};
>>>>>>> master
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use keyforge_physics::{EngineCompilationContext, EngineFactory};
    use keyforge_protocol::CostModelDto;
    use std::sync::Arc;

    fn mock_cost_model() -> CostModel {
        let json = r#"{
        "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
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
        let dto: CostModelDto = serde_json::from_str(json).unwrap();
        dto.into()
    }

    #[test]
    fn test_optimal_choice_monogram() {
        let keys = vec![
            KeyNode {
                index: KeyIndex(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex::new(0), // cost 100
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(1),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex::new(1), // cost 200
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap(),
        );
        let mut corpus_val = Corpus::default();
        let mut freqs = corpus_val.char_freqs.to_vec();
        freqs[97] = 1000;
        corpus_val.char_freqs = Arc::from(freqs);
        let corpus = Arc::new(corpus_val);
        let cm = Arc::new(mock_cost_model());
        let ctx = EngineCompilationContext {
            keyboard: kb,
            corpus,
            rubric: Arc::new(Rubric::default()),
            cost_model: cm,
            engine_config: keyforge_model::config::EngineConfig::default(),
        };

        let engine = EngineFactory::new_generic(&ctx).unwrap();
        let oracle = EngineFactory::new_exact(&ctx).unwrap();

        // Layout where 'a' is on BOTH keys
        let layout = Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(97)]);

        let score_engine = engine.score(&layout).unwrap();
        let score_oracle = oracle.score(&layout).unwrap();

        assert_eq!(score_engine, score_oracle);
    }

    #[test]
    fn test_optimal_choice_bigram() {
        let keys = vec![
            KeyNode {
                index: KeyIndex(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
<<<<<<< HEAD
                x: SpatialUnit::from_f32(0.0),
=======
                x: keyforge_model::types::SpatialUnit::from_f32(0.0),
>>>>>>> master
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(1),
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
<<<<<<< HEAD
                x: SpatialUnit::from_f32(1.0),
=======
                x: keyforge_model::types::SpatialUnit::from_f32(1.0),
>>>>>>> master
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(2),
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
<<<<<<< HEAD
                x: SpatialUnit::from_f32(10.0), // Far away
=======
                x: keyforge_model::types::SpatialUnit::from_f32(10.0), // Far away
>>>>>>> master
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap(),
        );
        let mut corpus_val = Corpus::default();
        corpus_val.bigrams = Arc::from(vec![(97, 98, 1000)]);
        let corpus = Arc::new(corpus_val);

        let rubric = Rubric::builder().travel_lat(1_000_000).build();
        let rubric_arc = Arc::new(rubric);

        let cm = Arc::new(mock_cost_model());
        let ctx = EngineCompilationContext {
            keyboard: kb,
            corpus,
            rubric: rubric_arc,
            cost_model: cm,
            engine_config: keyforge_model::config::EngineConfig::default(),
        };

        let engine = EngineFactory::new_generic(&ctx).unwrap();
        let oracle = EngineFactory::new_exact(&ctx).unwrap();

        // Layout where 'a' is at index 0, and 'b' is at BOTH index 1 and 2
        let layout =
            Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98), KeyCode::new(98)]);

        let score_engine = engine.score(&layout).unwrap();
        let score_oracle = oracle.score(&layout).unwrap();

        assert_eq!(score_engine, score_oracle);
    }

    #[test]
    fn test_optimal_choice_trigram() {
        let keys = vec![
            KeyNode {
                index: KeyIndex(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::PINKY,
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(1),
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex(2),
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                ..Default::default()
            },
            KeyNode {
                index: keyforge_model::types::KeyIndex(3),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap(),
        );
        let mut corpus_val = Corpus::default();
        corpus_val.trigrams = Arc::from(vec![(97, 98, 99, 1000)]);
        let corpus = Arc::new(corpus_val);

        let rubric = Rubric::builder()
            .roll_bonus(100_000_000)
            .redirect(500_000_000)
            .build();
        let rubric_arc = Arc::new(rubric);

        let cm = Arc::new(mock_cost_model());
        let ctx = EngineCompilationContext {
            keyboard: kb,
            corpus,
            rubric: rubric_arc,
            cost_model: cm,
            engine_config: keyforge_model::config::EngineConfig::default(),
        };

        let engine = EngineFactory::new_generic(&ctx).unwrap();
        let oracle = EngineFactory::new_exact(&ctx).unwrap();

        let layout = Layout::new_unchecked(vec![
            KeyCode::new(97),
            KeyCode::new(98),
            KeyCode::new(99),
            KeyCode::new(99),
        ]);

        let score_engine = engine.score(&layout).unwrap();
        let score_oracle = oracle.score(&layout).unwrap();

        assert_eq!(score_engine, score_oracle);
    }
}