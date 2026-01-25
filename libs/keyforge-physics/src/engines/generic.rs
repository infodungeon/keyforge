use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
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
    #[must_use]
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
            features: EngineFeatures::NONE,
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        crate::kernel::compute::state::with_scratch(|scratch| {
            self.score_with_scratch(layout, scratch)
        })
    }

    fn score_with_scratch(
        &self,
        layout: &Layout,
        scratch: &mut PhysicsScratch,
    ) -> Result<Score, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(Score(score_layout(&self.ctx, &validated, scratch)?))
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let layout_slice = validated.as_slice();

        crate::kernel::compute::state::with_scratch(|scratch| {
            let key_count = self.ctx.key_count;
            let (starts, counts, indices, offsets, used, _char_usage, _flat_map) =
                scratch.get_mut_scratch();
            let pm = PosMap::from_scratch(
                layout_slice,
                key_count,
                starts,
                counts,
                indices,
                offsets,
                used,
            );

            let eval_ctx = crate::kernel::EvaluationContext {
                engine: &self.ctx,
                pos_map: &pm,
            };

            // Access private kernels for breakdown
            let mono = crate::kernel::compute::scoring::score_monograms(&eval_ctx)?.0;
            let bigram = crate::kernel::compute::scoring::score_bigrams(&eval_ctx)?.0;
            let trigram = crate::kernel::compute::scoring::score_trigrams(&eval_ctx)?.0;

            // Clean up scratch for next use (score_layout usually does this, but we called sub-functions)
            scratch.clear_used();
            Ok((mono, bigram, trigram))
        })
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let pm = PosMap::from_slice(pos_map, self.ctx.key_count);

        calculate_swap_delta(&self.ctx, &validated, &pm, idx_a, idx_b)
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(crate::kernel::compute::analyze_layout(
            &self.ctx, &validated,
        ))
    }

    fn suggest_improvements(&self, layout: &Layout, include_thumbs: bool) -> Vec<SwapSuggestion> {
        crate::analysis::heuristics::suggest_swaps(&self.ctx, layout, include_thumbs)
    }

    fn context(&self) -> &EngineContext {
        &self.ctx
    }
}
