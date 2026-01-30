#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-evolution/tests/optimization_integration.rs
    //
    // Integration tests for the evolution module.
    // These tests exercise full optimization loops, cross-module orchestration,
    // and `ScoringEngine` usage (per ADR-015).

    use keyforge_evolution::{evolve, optimize};
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex, SpatialUnit};
    use keyforge_model::{
        Corpus, CostModel, EngineRequest, KeyNode, Keyboard, Rubric, SearchConfig,
    };
    use keyforge_physics::{EngineCompilationContext, EngineFactory, ScoringEngine};
    use std::sync::Arc;

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

    fn setup_env() -> (Arc<Keyboard>, Arc<Corpus>, Arc<Rubric>, Arc<CostModel>) {
        let keys = vec![
            KeyNode {
                index: 0,
                label: "k0".to_string(),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex::new(0),
                col: ColIndex::new(0),
                x: SpatialUnit::from_f32(0.0),
                y: SpatialUnit::from_f32(0.0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                label: "k1".to_string(),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(2),
                row: RowIndex::new(0),
                col: ColIndex::new(1),
                x: SpatialUnit::from_f32(1.0),
                y: SpatialUnit::from_f32(0.0),
                ..Default::default()
            },
            KeyNode {
                index: 2,
                label: "k2".to_string(),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(3),
                row: RowIndex::new(0),
                col: ColIndex::new(2),
                x: SpatialUnit::from_f32(2.0),
                y: SpatialUnit::from_f32(0.0),
                ..Default::default()
            },
        ];
        (
            Arc::new(Keyboard::new(keys, RowIndex::new(0), "test".into()).unwrap()),
            Arc::new(Corpus::default()),
            Arc::new(Rubric::default()),
            Arc::new(mock_cost_model()),
        )
    }

    #[test]
    fn test_legacy_optimize_entry_point() {
        let (kb, cp, rb, cm) = setup_env();
        let req = EngineRequest {
            keyboard: kb,
            corpus: cp,
            rubric: rb,
            cost_model: cm,
            engine_config: keyforge_model::config::EngineConfig::default(),
            config: SearchConfig::Annealing {
                steps: 10,
                start_temp: 10.0,
                end_temp: 1.0,
                seed: 123,
                patience: 100,
                reheats: 0,
                reheat_factor: 1.0,
                include_thumbs: false,
            },
            initial_layout: None,
            pinned_keys: vec![],
        };
        let result = optimize(&req).unwrap();
        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_evolve_api_direct() {
        let (kb, cp, rb, cm) = setup_env();
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard: kb.clone(),
            corpus: cp.clone(),
            rubric: rb.clone(),
            cost_model: cm.clone(),
            engine_config: keyforge_model::config::EngineConfig::default(),
        })
        .unwrap();
        let engine_arc: Arc<dyn ScoringEngine> = engine.into();
        let config = SearchConfig::Annealing {
            steps: 10,
            start_temp: 10.0,
            end_temp: 1.0,
            seed: 123,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
            include_thumbs: false,
        };
        let result = evolve(&engine_arc, &config, NoOpCallback, None, None).unwrap();
        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_pinned_key_swap() {
        let (kb, cp, rb, cm) = setup_env();
        let pinned = vec![Some(KeyCode::new(2)), None, None];
        let req = EngineRequest {
            keyboard: kb,
            corpus: cp,
            rubric: rb,
            cost_model: cm,
            engine_config: keyforge_model::config::EngineConfig::default(),
            config: SearchConfig::Annealing {
                steps: 10,
                start_temp: 10.0,
                end_temp: 1.0,
                seed: 123,
                patience: 100,
                reheats: 0,
                reheat_factor: 1.0,
                include_thumbs: false,
            },
            initial_layout: None,
            pinned_keys: pinned,
        };
        let result = optimize(&req).unwrap();
        assert_eq!(result.layout.keys()[0], KeyCode::new(2));
        assert_eq!(result.layout.keys()[2], KeyCode::new(0));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_oracle_pattern_match() {
        let (kb, cp, rb, cm) = setup_env();

        let config = SearchConfig::Annealing {
            steps: 2000,
            start_temp: 10.0,
            end_temp: 0.1,
            seed: 42,
            patience: 100,
            reheats: 0,
            reheat_factor: 1.0,
            include_thumbs: false,
        };

        let req = EngineRequest {
            keyboard: kb.clone(),
            corpus: cp.clone(),
            rubric: rb.clone(),
            cost_model: cm.clone(),
            engine_config: keyforge_model::config::EngineConfig::default(),
            config,
            initial_layout: None,
            pinned_keys: vec![],
        };

        let result = optimize(&req).unwrap();
        
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard: kb.clone(),
            corpus: cp.clone(),
            rubric: rb.clone(),
            cost_model: cm.clone(),
            engine_config: keyforge_model::config::EngineConfig::default(),
        }).unwrap();

        let scorer = keyforge_physics::verify::DeterministicScorer::new(engine.context().clone());
        let raw_score = scorer
            .score(&req.keyboard, &req.corpus, result.layout.keys())
            .expect("Oracle scoring failed");

        // Normalize logic from physics/lib.rs
        let raw_score_f32 = (raw_score as f32) / keyforge_model::constants::SCORE_SCALE;
        let total_freq: u64 = req.corpus.char_freqs.iter().sum();
        let norm_factor = if total_freq > 0 {
            100_000.0 / total_freq as f32
        } else {
            1.0
        };
        let final_reference = raw_score_f32 * norm_factor;

        assert!((result.score - final_reference).abs() < 1e-4);
    }

    struct NoOpCallback;
    impl keyforge_evolution::ProgressCallback for NoOpCallback {
        fn on_progress(
            &self,
            _epoch: usize,
            _score: f32,
            _layout: &[KeyCode],
            _ips: f32,
        ) -> keyforge_evolution::OptimizationControl {
            keyforge_evolution::OptimizationControl::Continue
        }
    }
}
