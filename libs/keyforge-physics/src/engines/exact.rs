use super::{EngineCapabilities, ScoringEngine};
use crate::verify::DeterministicScorer;
use crate::kernel::EngineContext;
use crate::kernel::compute::analyze_layout;
use crate::kernel::types::ValidatedLayout;
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Corpus, Keyboard, Layout, Rubric, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub struct ExactScoringEngine {
    scorer: DeterministicScorer,
    keyboard: Keyboard,
    corpus: Corpus,
    ctx: EngineContext,
}

impl ExactScoringEngine {
    pub fn new(keyboard: Keyboard, corpus: Corpus, rubric: Rubric, cost_model: &keyforge_model::CostModel, ctx: EngineContext) -> Self {
        let scorer = DeterministicScorer::new(&rubric, cost_model);
        Self {
            scorer,
            keyboard,
            corpus,
            ctx,
        }
    }
}

impl ScoringEngine for ExactScoringEngine {
    fn name(&self) -> &'static str {
        "Exact (Oracle)"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: true,
            supports_avx2: false,
            supports_blocking: false,
        }
    }

    fn key_count(&self) -> usize {
        self.keyboard.keys.len()
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        self.scorer.score(&self.keyboard, &self.corpus, layout.keys.as_slice()).map(Score)
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        self.scorer.score_detailed(&self.keyboard, &self.corpus, layout.keys.as_slice())
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let current = self.score(layout)?;
        
        let mut swapped_keys = layout.keys.clone();
        swapped_keys.swap(idx_a, idx_b);
        let swapped = Layout::new_unchecked(swapped_keys);
        
        let new_score = self.score(&swapped)?;
        
        let diff = (new_score.0 as i128) - (current.0 as i128);
        Ok(diff.clamp(i64::MIN as i128, i64::MAX as i128) as i64)
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(analyze_layout(&self.ctx, &validated))
    }

        fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion> {

            crate::analysis::heuristics::suggest_swaps(&self.ctx, layout, include_thumbs)

        }

    

        fn context(&self) -> &EngineContext {
        &self.ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineFactory;
    use keyforge_model::{KeyNode, Keyboard, Corpus, Rubric, CostModel};
    use keyforge_model::types::{KeyCode, HandIndex, FingerIndex};
    use std::collections::HashMap;

    fn setup_minimal() -> (Keyboard, Corpus, Rubric, CostModel) {
        let keys = vec![
            KeyNode { index: 0, hand: HandIndex::LEFT, finger: FingerIndex::INDEX, ..Default::default() },
            KeyNode { index: 1, hand: HandIndex::LEFT, finger: FingerIndex::MIDDLE, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0).unwrap();
        let mut corpus = Corpus::default();
        corpus.char_freqs[97] = 100;
        corpus.char_freqs[98] = 200;
        corpus.bigrams.push((97, 98, 50));
        
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: HashMap::new(),
        });

        (kb, corpus, Rubric::default(), cm)
    }

    #[test]
    fn test_exact_engine_trait_methods() {
        let (kb, corpus, rubric, cm) = setup_minimal();
        let engine = EngineFactory::new_exact(&kb, &corpus, &rubric, &cm).unwrap();
        
        assert_eq!(engine.name(), "Exact (Oracle)");
        assert!(engine.capabilities().is_exact);
        assert!(!engine.capabilities().supports_avx2);
        assert_eq!(engine.key_count(), 2);
        
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98)]);
        
        // score and score_detailed
        let score = engine.score(&layout).unwrap();
        let detailed = engine.score_detailed(&layout).unwrap();
        assert_eq!(score.0, detailed.0 + detailed.1 + detailed.2);
        
        // calculate_swap_delta
        let pos_map = vec![0, 1]; // Simplified
        let delta = engine.calculate_swap_delta(&layout, &pos_map, 0, 1).unwrap();
        
        let mut swapped_keys = layout.keys.clone();
        swapped_keys.swap(0, 1);
        let swapped_layout = Layout::new_unchecked(swapped_keys);
        let score_after = engine.score(&swapped_layout).unwrap();
        assert_eq!(delta, score_after.0 - score.0);
        
        // analyze
        let report = engine.analyze(&layout).unwrap();
        assert!(report.score > 0.0);
        
        // suggest_improvements
        let suggestions = engine.suggest_improvements(&layout, true);
        // Might be empty if already optimal or too small, but we call it for coverage
        let _ = suggestions.len();
        
        // context
        let _ = engine.context();
    }
}

    