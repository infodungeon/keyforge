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
            supports_avx2: false,
            supports_blocking: false,
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        std::thread_local! {
            static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new());
        }
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            Ok(Score(score_layout(&self.ctx, &validated, &mut s)?))
        })
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        std::thread_local! {
            static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new());
        }
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let layout_slice = validated.as_slice();
        SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            let key_count = self.ctx.key_count;
            let (starts, counts, indices, offsets, used) = s.get_mut_scratch();
            let pm = PosMap::from_scratch(
                layout_slice,
                key_count,
                starts,
                counts,
                indices,
                offsets,
                used,
            );

            // Access private kernels for breakdown
            let mono = crate::kernel::compute::scoring::score_monograms(&self.ctx, &pm)?.0;
            let bigram = crate::kernel::compute::scoring::score_bigrams(&self.ctx, &pm)?.0;
            let trigram = crate::kernel::compute::scoring::score_trigrams(&self.ctx, &pm)?.0;

            // Clean up scratch for next use (score_layout usually does this, but we called sub-functions)
            s.clear_used();
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

        // Task-phys-rev-039: Reuse provided pos_map if possible
        if pos_map.len() >= 65536 {
            // In KeyForge, a 'full' pos_map is 65536 entries.
            // If the optimizer provides one, we can wrap it.
            // We need to implement PosMap::from_slice for this.
            // For now, let's assume we still need the full scratch-based PosMap
            // until PosMap is refactored to support slices.
        }

        std::thread_local! {
            static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new());
        }

        SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            let key_count = self.ctx.key_count;
            let (starts, counts, indices, offsets, used) = s.get_mut_scratch();
            let pm = PosMap::from_scratch(
                validated.as_slice(),
                key_count,
                starts,
                counts,
                indices,
                offsets,
                used,
            );

            let delta = calculate_swap_delta(&self.ctx, &validated, &pm, idx_a, idx_b)?;
            s.clear_used();
            Ok(delta)
        })
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
