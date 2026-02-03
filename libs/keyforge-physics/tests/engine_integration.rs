// libs/keyforge-physics/tests/engine_integration.rs

#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
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

        let layout =
            Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98), KeyCode::new(99)]);
        let score = engine.score(&layout).unwrap();
        let (m, b, t) = engine.score_detailed(&layout).unwrap();
        assert_eq!(score.raw(), m + b + t);
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
        assert_eq!(engine.key_count(), 3);
    }
}