// libs/keyforge-physics/src/engines/arm_neon.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

/// Configuration for the ARM-specific scoring engine.
#[derive(Debug, Clone, Copy)]
pub struct ArmNeonConfig {
    pub l1d_size_bytes: usize,
    pub l2_size_bytes: usize,
}

impl Default for ArmNeonConfig {
    fn default() -> Self {
        Self {
            l1d_size_bytes: 32 * 1024,
            l2_size_bytes: 512 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArmNeonScoringEngine {
    pub(crate) ctx: EngineContext,
    _config: ArmNeonConfig,
}

impl ArmNeonScoringEngine {
    #[must_use]
    pub fn new(ctx: EngineContext, config: Option<ArmNeonConfig>) -> Self {
        Self {
            ctx,
            _config: config.unwrap_or_default(),
        }
    }
}

impl ScoringEngine for ArmNeonScoringEngine {
    fn name(&self) -> &'static str {
        "ARM NEON Optimized"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures {
                supports_avx2: false,
                supports_neon: true,
                supports_blocking: true,
            },
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        std::thread_local! {
            static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new());
        }
        SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            self.score_with_scratch(layout, &mut s)
        })
    }

    fn score_with_scratch(
        &self,
        layout: &Layout,
        scratch: &mut PhysicsScratch,
    ) -> Result<Score, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        #[cfg(target_arch = "aarch64")]
        {
            // Task-phys-neon-001: Implement real NEON kernel.
            // For now, use scalar fallback.
            return score_layout_scalar(&self.ctx, &validated, scratch).map(Score);
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            score_layout_scalar(&self.ctx, &validated, scratch).map(Score)
        }
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
            let (starts, counts, indices, offsets, used, _char_usage) = s.get_mut_scratch();
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

            let mono = crate::kernel::compute::scoring::score_monograms(&eval_ctx)?.0;
            let bigram = crate::kernel::compute::scoring::score_bigrams(&eval_ctx)?.0;
            let trigram = crate::kernel::compute::scoring::score_trigrams(&eval_ctx)?.0;
            s.clear_used();
            Ok((mono, bigram, trigram))
        })
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _pos_map: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        std::thread_local! {
            static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new());
        }
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;

        SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            let key_count = self.ctx.key_count;
            let (starts, counts, indices, offsets, used, _char_usage) = s.get_mut_scratch();
            let pm = PosMap::from_scratch(
                validated.as_slice(),
                key_count,
                starts,
                counts,
                indices,
                offsets,
                used,
            );

            let delta = crate::kernel::compute::calculate_swap_delta(
                &self.ctx, &validated, &pm, idx_a, idx_b,
            );
            s.clear_used();
            delta
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

fn score_layout_scalar(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    crate::kernel::compute::score_layout(ctx, layout, scratch)
}

#[cfg(test)]
mod tests {}
