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

