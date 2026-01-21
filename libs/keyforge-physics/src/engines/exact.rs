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

