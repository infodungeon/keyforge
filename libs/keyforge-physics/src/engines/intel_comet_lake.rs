#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{flow::calculate_flow_cost, PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

/// Configuration for the Intel-specific scoring engine.
#[derive(Debug, Clone, Copy)]
pub struct IntelEngineConfig {
    pub l1d_size_bytes: usize,
    pub l2_size_bytes: usize,
    pub l3_size_bytes: usize,
    /// If true, use prefetching intrinsics.
    pub use_prefetch: bool,
}

impl Default for IntelEngineConfig {
    fn default() -> Self {
        Self {
            l1d_size_bytes: 32 * 1024,
            l2_size_bytes: 256 * 1024,
            l3_size_bytes: 8 * 1024 * 1024,
            use_prefetch: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntelScoringEngine {
    pub(crate) ctx: EngineContext,
    config: IntelEngineConfig,
}

impl IntelScoringEngine {
    #[must_use]
    pub fn new(ctx: EngineContext, config: Option<IntelEngineConfig>) -> Self {
        Self {
            ctx,
            config: config.unwrap_or_default(),
        }
    }

    fn score_internal(&self, layout: &Layout, force_scalar: bool) -> Result<Score, PhysicsError> {
        std::thread_local! {
            static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new());
        }
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;

        SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                if is_x86_feature_detected!("avx2") && !force_scalar {
                    unsafe {
                        score_layout_avx2(&self.ctx, &validated, &mut s, &self.config).map(Score)
                    }
                } else {
                    score_layout_scalar(&self.ctx, &validated, &mut s).map(Score)
                }
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                score_layout_scalar(&self.ctx, &validated, &mut s).map(Score)
            }
        })
    }
}

impl ScoringEngine for IntelScoringEngine {
    fn name(&self) -> &'static str {
        "Intel Comet Lake (AVX2 Optimized)"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures {
                supports_avx2: true,
                supports_neon: false,
                supports_blocking: true,
            },
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        self.score_internal(layout, false)
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

            let eval_ctx = crate::kernel::EvaluationContext {
                engine: &self.ctx,
                pos_map: &pm,
            };

            let mono = crate::kernel::compute::scoring::score_monograms(&eval_ctx)?.0;
            let bigram = crate::kernel::compute::scoring::score_bigrams(&eval_ctx)?.0;
            let trigram = crate::kernel::compute::scoring::score_trigrams(&eval_ctx)?.0;
            // cleanup
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

// -----------------------------------------------------------------------------
// Scalar Fallback (Copy of Generic)
// -----------------------------------------------------------------------------

#[allow(clippy::cast_possible_wrap)]
fn score_layout_scalar(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    crate::kernel::compute::score_layout(ctx, layout, scratch)
}

// -----------------------------------------------------------------------------
// AVX2 Optimized
// -----------------------------------------------------------------------------

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[allow(clippy::cast_possible_wrap)]
unsafe fn score_layout_avx2(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
    _config: &IntelEngineConfig,
) -> Result<i64, PhysicsError> {
    // Task-phys-rev-032: Real AVX2 implementation pending.
    // Fallback to scalar for now to ensure bit-perfect accuracy.
    score_layout_scalar(ctx, layout, scratch)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Intent: Verify `IntelEngineConfig` default values are sensible.
    /// Expected: Default config has prefetch enabled and L1D cache size of 32KB.
    #[test]
    fn test_config_defaults() {
        let config = IntelEngineConfig::default();
        assert!(config.use_prefetch);
        assert_eq!(config.l1d_size_bytes, 32 * 1024);
    }
}
