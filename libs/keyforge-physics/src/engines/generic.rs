use super::{EngineCapabilities, ScoringEngine};
use crate::kernel::compute::{calculate_swap_delta, score_layout, PhysicsScratch, PosMap};
use crate::kernel::types::ValidatedLayout;
use crate::kernel::EngineContext;
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub struct GenericScoringEngine {
    pub(crate) ctx: EngineContext,
}

impl GenericScoringEngine {
    pub fn new(ctx: EngineContext) -> Self {
        Self { ctx }
    }
}

impl ScoringEngine for GenericScoringEngine {
    fn name(&self) -> &'static str {
        "Generic Optimized"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            supports_avx2: false,
            supports_blocking: false,
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        let mut scratch = Box::new(PhysicsScratch::new());
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(Score(score_layout(&self.ctx, &validated, &mut scratch)?))
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let mut scratch = Box::new(PhysicsScratch::new());
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let layout_slice = validated.as_slice();
        let pm = PosMap::from_scratch(
            layout_slice,
            self.ctx.key_count,
            scratch.starts.as_mut_slice(),
            scratch.counts.as_mut_slice(),
            scratch.indices.as_mut_slice(),
            scratch.current_offsets.as_mut_slice(),
            &mut scratch.used_keys,
        );

        // Access private kernels for breakdown
        let mono = crate::kernel::compute::scoring::score_monograms(&self.ctx, &pm)?.0;
        let bigram = crate::kernel::compute::scoring::score_bigrams(&self.ctx, &pm)?.0;
        let trigram = crate::kernel::compute::scoring::score_trigrams(&self.ctx, &pm)?.0;
        Ok((mono, bigram, trigram))
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let mut scratch = Box::new(PhysicsScratch::new());
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;

        let pm = PosMap::from_scratch(
            validated.as_slice(),
            self.ctx.key_count,
            scratch.starts.as_mut_slice(),
            scratch.counts.as_mut_slice(),
            scratch.indices.as_mut_slice(),
            scratch.current_offsets.as_mut_slice(),
            &mut scratch.used_keys,
        );

        Ok(calculate_swap_delta(&self.ctx, &validated, &pm, idx_a, idx_b))
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(crate::kernel::compute::analyze_layout(&self.ctx, &validated))
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
    fn test_generic_engine_trait_methods() {
        let (kb, corpus, rubric, cm) = setup_minimal();
        let engine = EngineFactory::new_generic(&kb, &corpus, &rubric, &cm).unwrap();
        
        assert_eq!(engine.name(), "Generic Optimized");
        assert!(!engine.capabilities().is_exact);
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
        let _ = suggestions.len();
        
        // context
        let _ = engine.context();
    }
}