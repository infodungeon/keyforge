// libs/keyforge-physics/src/kernel/compute/tests.rs

#[keyforge_testing_macros::kf_test]
mod unit_tests {
    use crate::error::PhysicsError;
    use crate::kernel::compute::{analyze_layout, calculate_swap_delta, PosMap};
    use crate::kernel::types::ValidatedLayout;
    use crate::{EngineContext, EngineFactory, ScoringContext, ScoringEngine, Uncompiled};
    use keyforge_model::testing::setup_minimal_assets;
    use keyforge_model::{
        types::{FingerIndex, HandIndex, KeyCode, RowIndex},
        Corpus, CostModel, KeyNode, Keyboard, Layout, MetricId, Rubric,
    };
    use std::sync::Arc;

    fn setup_kb_robust() -> Keyboard {
        let keys: Vec<KeyNode> = (0..5)
            .map(|i| KeyNode {
                index: i,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(i as u8),
                x: i as f32,
                ..Default::default()
            })
            .collect();
        Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap()
    }

    fn mock_cost_model() -> CostModel {
        let json = r#"{
            "version": "2.0", "description": "Test", "unit": "pts",
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

    fn setup_test_ctx(
        kb: Keyboard,
        corpus: Corpus,
        rubric: Rubric,
        cm: CostModel,
    ) -> EngineContext {
        let ctx = ScoringContext::new(
            Arc::new(kb),
            Arc::new(corpus),
            Arc::new(rubric),
            Arc::new(cm),
            keyforge_model::config::EngineConfig::default(),
        )
        .compile()
        .unwrap();
        ctx.state.inner
    }

    #[test]
    fn test_math_boundaries_infinity() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ]);
        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(97, 98, 1000)].into();

        let rubric = Rubric::builder().travel_lat(f32::INFINITY).build();

        let cost_model = mock_cost_model();
        // Compilation or scoring should fail gracefully
        let res = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(rubric.clone()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ));
        if let Ok(engine) = res {
            let score_res = engine.score(&layout);
            assert!(
                score_res.is_err(),
                "Scoring should fail with INFINITY travel cost"
            );
        }
    }

    #[test]
    fn test_math_boundaries_nan() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ]);
        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(97, 98, 1000)].into();

        let rubric = Rubric::builder().travel_lat(f32::NAN).build();

        let cost_model = mock_cost_model();
        // Compilation or scoring should fail gracefully
        let res = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(rubric.clone()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ));
        if let Ok(engine) = res {
            let score_res = engine.score(&layout);
            assert!(
                score_res.is_err(),
                "Scoring should fail with NAN travel cost"
            );
        }
    }

    #[test]
    fn test_saturation_protection() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ]);
        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(97, 98, u32::MAX)].into();

        let rubric = Rubric::builder().travel_lat(1_000_000.0).build();

        let cost_model = mock_cost_model();
        let res = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(rubric.clone()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ));
        if let Ok(engine) = res {
            let score_res = engine.score(&layout);
            assert!(
                matches!(score_res, Err(PhysicsError::ScoreOverflow { .. })),
                "Should return ScoreOverflow error instead of panicking"
            );
        }
    }

    #[test]
    fn test_missing_keys_in_layout() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![
            KeyCode(97),
            KeyCode(0),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ]);
        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(97, 98, 100)].into();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(Rubric::default()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();
        let score = engine.score(&layout).unwrap().to_f32();
        assert_eq!(score, 0.0);
    }

    fn setup_engine() -> Box<dyn ScoringEngine> {
        let kb = setup_kb_robust();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let cost_model = mock_cost_model();
        EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb),
            Arc::new(corpus),
            Arc::new(rubric),
            Arc::new(cost_model),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap()
    }

    #[test]
    fn test_validated_layout_construction() {
        let engine = setup_engine();
        let layout = Layout::new_unchecked(vec![
            KeyCode(10),
            KeyCode(20),
            KeyCode(30),
            KeyCode(40),
            KeyCode(50),
        ]);

        let validated = ValidatedLayout::new(layout.keys(), engine.key_count()).unwrap();
        assert_eq!(validated.as_slice().len(), 5);

        let sub_slice = &layout.keys()[0..3];
        let validated_partial = ValidatedLayout::new(sub_slice, 3).unwrap();
        assert_eq!(validated_partial.as_slice().len(), 3);

        // Underflow
        let bad_layout = Layout::new_unchecked(vec![KeyCode(0)]);
        let res = ValidatedLayout::new(bad_layout.keys(), engine.key_count());
        assert!(res.is_err());
    }

    #[test]
    fn test_validated_layout_mismatch() {
        let engine = setup_engine();
        let layout = Layout::new_unchecked(vec![]);
        let validated_res = ValidatedLayout::new(layout.keys(), engine.key_count());
        assert!(validated_res.is_err());
    }

    #[test]
    fn test_swap_delta_bounds() {
        let kb = setup_kb_robust();
        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(97, 98, 100)].into();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(Rubric::default()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();

        let validated = ValidatedLayout::new(
            &[
                KeyCode(97),
                KeyCode(98),
                KeyCode(99),
                KeyCode(100),
                KeyCode(101),
            ],
            engine.key_count(),
        )
        .unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [keyforge_model::types::KeyIndex::new(0); 512];
        let mut current_offsets = [0u8; 65536];
        let mut used_keys_scratch = Vec::new();
        let pm = PosMap::from_scratch(
            &[
                KeyCode(97),
                KeyCode(98),
                KeyCode(99),
                KeyCode(100),
                KeyCode(101),
            ],
            engine.key_count(),
            &mut starts,
            &mut counts,
            &mut indices,
            &mut current_offsets,
            &mut used_keys_scratch,
        );

        let delta_res = calculate_swap_delta(engine.context(), &validated, &pm, 0, 100);
        assert!(delta_res.is_err());
    }

    #[test]
    fn test_analyze_layout_empty() {
        let kb = setup_kb_robust();
        let corpus = Corpus::default();
        let rubric = Rubric::default();
        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(rubric.clone()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();

        let validated_res = ValidatedLayout::new(&[], engine.key_count());
        assert!(validated_res.is_err());
    }

    #[test]
    fn test_delta_internals_manual() {
        let keys = vec![
            KeyNode {
                index: 0,
                x: 0.0,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                x: 10.0,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                x: 20.0,
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap();

        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(0, 1, 100)].into();
        corpus.trigrams = vec![(0, 1, 2, 100)].into();

        let rubric = Rubric::builder().travel_lat(1.0).build();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(rubric.clone()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();

        let layout_keys = vec![KeyCode(0), KeyCode(1), KeyCode(2)];
        let layout = Layout::new_unchecked(layout_keys.clone());
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [keyforge_model::types::KeyIndex::new(0); 512];
        let mut current_offsets = [0u8; 65536];
        let mut used_keys_scratch = Vec::new();
        let pm = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            &mut starts,
            &mut counts,
            &mut indices,
            &mut current_offsets,
            &mut used_keys_scratch,
        );

        let score_before = engine.score(&layout).unwrap().0;
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 1, 2).unwrap();

        let mut next_keys = layout_keys.clone();
        next_keys.swap(1, 2);
        let swapped_layout = Layout::new_unchecked(next_keys);
        let score_after = engine.score(&swapped_layout).unwrap().0;

        assert_eq!(
            score_after - score_before,
            delta,
            "Manual delta check failed"
        );
    }

    #[test]
    fn test_delta_self_loop() {
        let keys = vec![
            KeyNode {
                index: 0,
                x: 0.0,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                x: 10.0,
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap();

        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(0, 0, 100)].into();

        let rubric = Rubric::builder().travel_lat(1.0).trigram_limit(0).build();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(rubric.clone()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();

        let layout_keys = vec![KeyCode(0), KeyCode(1)];
        let layout = Layout::new_unchecked(layout_keys.clone());
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [keyforge_model::types::KeyIndex::new(0); 512];
        let mut current_offsets = [0u8; 65536];
        let mut used_keys_scratch = Vec::new();
        let pm = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            &mut starts,
            &mut counts,
            &mut indices,
            &mut current_offsets,
            &mut used_keys_scratch,
        );

        let score_before = engine.score(&layout).unwrap().0;
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 1).unwrap();

        let mut next_keys = layout_keys.clone();
        next_keys.swap(0, 1);
        let swapped_layout = Layout::new_unchecked(next_keys);
        let score_after = engine.score(&swapped_layout).unwrap().0;

        assert_eq!(
            score_after - score_before,
            delta,
            "Self loop delta check failed"
        );
    }

    #[test]
    fn test_delta_missing_candidates() {
        let kb = setup_kb_robust();
        let mut corpus = Corpus::default();
        // Bigram with a char not in layout
        corpus.bigrams = vec![(97, 255, 100)].into();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(Rubric::default()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();
        let layout_keys = vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ];
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();

        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [keyforge_model::types::KeyIndex::new(0); 512];
        let mut current_offsets = [0u8; 65536];
        let mut used_keys_scratch = Vec::new();
        let pm = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            &mut starts,
            &mut counts,
            &mut indices,
            &mut current_offsets,
            &mut used_keys_scratch,
        );

        // This should not panic and should handle missing candidates
        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 1).unwrap();
        assert_eq!(delta, 0);
    }

    #[test]
    fn test_delta_trigram_overlaps() {
        let keys: Vec<KeyNode> = (0..3)
            .map(|i| KeyNode {
                index: i,
                ..Default::default()
            })
            .collect();
        let kb = Keyboard::new(keys, keyforge_model::types::RowIndex(0), "test".into()).unwrap();
        let mut corpus = Corpus::default();
        // Trigrams like (a,a,x), (b,a,x), (x,a,a), etc.
        corpus.trigrams = vec![(97, 97, 98, 100), (98, 97, 99, 100), (97, 98, 97, 100)].into();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(Rubric::default()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();
        let layout_keys = vec![KeyCode(97), KeyCode(98), KeyCode(99)];
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();

        let mut starts = [0u16; 65536];
        let mut counts = [0u8; 65536];
        let mut indices = [keyforge_model::types::KeyIndex::new(0); 512];
        let mut current_offsets = [0u8; 65536];
        let mut used_keys_scratch = Vec::new();
        let pm = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            &mut starts,
            &mut counts,
            &mut indices,
            &mut current_offsets,
            &mut used_keys_scratch,
        );

        let delta = calculate_swap_delta(engine.context(), &validated, &pm, 0, 1).unwrap();
        // We just care that it executes the continue branches
        assert!(delta != i64::MAX);
    }

    #[test]
    fn test_score_bigram_modifier_overflow() {
        let kb = setup_kb_robust();
        let mut corpus = Corpus::default();
        corpus.bigrams = vec![(97, 98, 100)].into();

        let cost_model = mock_cost_model();
        let engine = EngineFactory::new_generic(&ScoringContext::new(
            Arc::new(kb.clone()),
            Arc::new(corpus.clone()),
            Arc::new(Rubric::default()),
            Arc::new(cost_model.clone()),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();
        let mut ctx = engine.context().clone();
        // Huge modifier
        ctx.sequence_modifiers = std::sync::Arc::new(std::collections::HashMap::from([(
            (97, 98),
            crate::kernel::types::Score(i64::MAX / 2),
        )]));

        // Use a mutable copy of the geometry to inject the huge cost
        let mut geom = ctx.geometry.clone();
        let mut costs = (*geom.cost_matrix).to_vec();
        // Inject a massive cost into the matrix for a pair (0, 1)
        // costs[row * width + col]
        #[allow(clippy::erasing_op)]
        {
            costs[0 * ctx.key_count + 1] = crate::kernel::types::Score(i64::MAX / 2);
        }
        geom.cost_matrix = costs.into();
        ctx.geometry = geom;

        let layout_keys = vec![
            KeyCode(97),
            KeyCode(98),
            KeyCode(99),
            KeyCode(100),
            KeyCode(101),
        ];
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        let mut scratch = crate::kernel::compute::PhysicsScratch::try_new().unwrap();

        let res = crate::kernel::compute::score_layout(&ctx, &validated, &mut scratch);
        assert!(matches!(res, Err(PhysicsError::ScoreOverflow { .. })));
    }

    #[test]
    fn test_analyze_layout_basic() {
        let (kb, mut corpus, rubric, cost_model) = setup_minimal_assets();
        // Ensure bigrams exist
        corpus.bigrams = vec![(97, 98, 1000)].into();

        let ctx = setup_test_ctx(kb.clone(), corpus.clone(), rubric.clone(), cost_model);
        let layout_keys = vec![KeyCode(97), KeyCode(98), KeyCode(99)];
        let validated = ValidatedLayout::new(&layout_keys, kb.count()).unwrap();

        let report = analyze_layout(&ctx, &validated).unwrap();
        assert!(report.summary.score > 0.0);
    }

    #[test]
    fn test_analyze_layout_duplicate_keys() {
        let mut keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys.clone(), RowIndex(0), "test".into()).unwrap();
        let (mut corpus, rubric, cost_model) =
            (Corpus::default(), Rubric::default(), mock_cost_model());
        corpus.char_freqs = Arc::from({
            let mut f = vec![0; 65536];
            f[97] = 1000;
            f
        });

        let ctx = ScoringContext::new(
            Arc::new(kb),
            Arc::new(corpus),
            Arc::new(rubric),
            Arc::new(cost_model),
            keyforge_model::config::EngineConfig::default(),
        )
        .compile()
        .unwrap();
        let layout_keys = vec![KeyCode(97), KeyCode(97)];
        let layout = Layout::new_unchecked(layout_keys.clone());

        let report = analyze_layout(
            &ctx.state.inner,
            &ValidatedLayout::new(&layout_keys, 2).unwrap(),
        )
        .unwrap();
        assert!(report.summary.score > 0.0);
    }
}
