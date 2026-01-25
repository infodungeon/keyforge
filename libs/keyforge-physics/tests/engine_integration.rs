#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-physics/tests/engine_integration.rs

    use keyforge_model::testing::setup_minimal_assets;
    use keyforge_model::{KeyCode, Layout};
    use keyforge_physics::{EngineCompilationContext, EngineFactory};
    use std::sync::Arc;

    #[test]
    fn test_generic_engine_trait_methods() {
        let (kb, corpus, rubric, cm) = setup_minimal_assets();
        let engine = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard: Arc::new(kb),
            corpus: Arc::new(corpus),
            rubric: Arc::new(rubric),
            cost_model: Arc::new(cm),
            engine_config: keyforge_model::config::EngineConfig::default(),
        })
        .unwrap();

        assert_eq!(engine.name(), "Generic Optimized");
        assert!(!engine.capabilities().is_exact);
        assert_eq!(engine.key_count(), 3);

        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99)]);
        let score = engine.score(&layout).unwrap();
        let detailed = engine.score_detailed(&layout).unwrap();
        assert_eq!(score.0, detailed.0 + detailed.1 + detailed.2);
    }

    #[test]
    fn test_exact_engine_trait_methods() {
        let (kb, corpus, rubric, cm) = setup_minimal_assets();
        let engine = EngineFactory::new_exact(&EngineCompilationContext {
            keyboard: Arc::new(kb),
            corpus: Arc::new(corpus),
            rubric: Arc::new(rubric),
            cost_model: Arc::new(cm),
            engine_config: keyforge_model::config::EngineConfig::default(),
        })
        .unwrap();

        assert_eq!(engine.name(), "Exact (Oracle)");
        assert!(engine.capabilities().is_exact);
        assert!(!engine
            .capabilities()
            .features
            .contains(keyforge_physics::EngineFeatures::AVX2));
        assert_eq!(engine.key_count(), 3);
    }

    #[test]
    fn test_intel_engine_trait_methods() {
        let (kb, corpus, rubric, cm) = setup_minimal_assets();
        let engine = EngineFactory::new_intel_comet_lake(
            &EngineCompilationContext {
                keyboard: Arc::new(kb),
                corpus: Arc::new(corpus),
                rubric: Arc::new(rubric),
                cost_model: Arc::new(cm),
                engine_config: keyforge_model::config::EngineConfig::default(),
            },
            None,
        )
        .unwrap();

        assert_eq!(engine.name(), "Intel Comet Lake (AVX2 Optimized)");
        // bitflags API: contains()
        assert!(engine
            .capabilities()
            .features
            .contains(keyforge_physics::EngineFeatures::AVX2));
        assert_eq!(engine.key_count(), 3);
    }
}
