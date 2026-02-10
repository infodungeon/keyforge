// libs/keyforge-physics/src/kernel/compute/tests.rs

use super::*;
use crate::engines::ScoringEngine;
use crate::kernel::compiler::Compiler;
use crate::kernel::types::{KeyCode, KeyIndex, RowIndex, Score};
use crate::PhysicsError;
use keyforge_model::testing::mock_cost_model;
use keyforge_model::{Corpus, KeyNode, Keyboard, Layout, Rubric};
use std::sync::Arc;

fn setup_kb_robust() -> Keyboard {
    let keys: Vec<KeyNode> = (0..5)
        .map(|i| KeyNode {
            index: KeyIndex::new(i as u16),
            hand: keyforge_model::types::HandIndex::new(0),
            finger: keyforge_model::types::FingerIndex::new(i as u8),
            x: keyforge_model::types::SpatialUnit::from_f32(i as f32),
            ..Default::default()
        })
        .collect();
    Keyboard::new(keys, RowIndex::new(0), "test".into()).expect("Failed to create keyboard")
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::EngineCompilationContext;
    use crate::EngineFactory;

    #[test]
    #[should_panic]
    fn test_math_boundaries_overflow() {
        let _ = Rubric::builder().travel_lat(i64::MAX).build();
    }

    #[test]
    fn test_saturation_protection() {
        let kb = setup_kb_robust();
        let layout = Layout::new_unchecked(vec![
            KeyCode::new(97),
            KeyCode::new(98),
            KeyCode::new(99),
            KeyCode::new(100),
            KeyCode::new(101),
        ]);
        let mut corpus = Corpus::default();
        // Use codes 97 ('a') and 101 ('e') which are at indices 0 and 4 in the layout (dist = 4000)
        corpus.bigrams = Arc::from(vec![(97, 101, u32::MAX)]);

        // Max possible weight in FixedWeight is ~2.1M
        let rubric = Rubric::builder().travel_lat(2_000_000).build();

        let cost_model = mock_cost_model();
        let res = EngineFactory::new_generic(&EngineCompilationContext {
            keyboard: Arc::new(kb.clone()),
            corpus: Arc::new(corpus.clone()),
            rubric: Arc::new(rubric.clone()),
            cost_model: Arc::new(cost_model.clone()),
            engine_config: keyforge_model::config::EngineConfig::default(),
        });
        if let Ok(engine) = res {
            let score_res = engine.score(&layout);
            assert!(
                matches!(score_res, Err(PhysicsError::ScoreOverflow { .. })),
                "Should return ScoreOverflow error instead of panicking"
            );
        }
    }

    #[test]
    fn test_score_overflow_edge_cases() {
        let mut ctx = Compiler::compile(
            &setup_kb_robust(),
            &Corpus::default(),
            &Rubric::default(),
            &mock_cost_model(),
        )
        .unwrap();

        // 1. Mono overflow
        {
            let mut key_costs = (*ctx.geometry.key_costs).to_vec();
            for cost in &mut key_costs {
                *cost = Score::from_scaled_i64(i64::MAX / 2);
            }
            ctx.geometry.key_costs = Arc::from(key_costs);
            let mut freqs = [0u64; 65536];
            freqs[97] = 3;
            ctx.corpus.char_freqs = Arc::from(freqs);

            let engine = crate::engines::generic::GenericScoringEngine::new(ctx.clone());
            let layout = Layout::new_unchecked(vec![KeyCode::new(97); 5]);
            let res = engine.score(&layout);
            assert!(matches!(res, Err(PhysicsError::ScoreOverflow { .. })));
        }

        // 2. Bigram overflow
        {
            let mut costs = (*ctx.geometry.cost_matrix).to_vec();
            costs[1] = Score::from_scaled_i64(i64::MAX / 2);
            ctx.geometry.cost_matrix = Arc::from(costs);
            ctx.corpus.bigram_starts = Arc::from(vec![0, 1, 1, 1, 1, 1]);
            ctx.corpus.bigram_others = Arc::from(vec![KeyCode::new(98)]);
            ctx.corpus.bigram_freqs = Arc::from(vec![3]);

            let engine = crate::engines::generic::GenericScoringEngine::new(ctx);
            let layout = Layout::new_unchecked(vec![
                KeyCode::new(97),
                KeyCode::new(98),
                KeyCode::new(99),
                KeyCode::new(100),
                KeyCode::new(101),
            ]);
            assert!(matches!(
                engine.score(&layout),
                Err(PhysicsError::ScoreOverflow { .. })
            ));
        }
    }
}
