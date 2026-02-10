// libs/keyforge-physics/src/engines/arm_sve.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::config::EngineConfig;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub(crate) struct ArmSveScoringEngine {
    pub(crate) ctx: EngineContext,
    _config: EngineConfig,
}

impl ArmSveScoringEngine {
    #[must_use]
    pub(crate) fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
        Self {
            ctx,
            _config: config.unwrap_or_default(),
        }
    }
}

impl ScoringEngine for ArmSveScoringEngine {
    fn name(&self) -> &'static str {
        "ARM SVE Optimized"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures::SVE | EngineFeatures::BLOCKING,
        }
    }

    fn key_count(&self) -> usize {
        self.ctx.key_count
    }

    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        crate::kernel::compute::state::with_scratch(|scratch| {
            self.score_with_scratch(layout, scratch)
        })?
    }

    fn score_with_scratch(
        &self,
        layout: &Layout,
        scratch: &mut PhysicsScratch,
    ) -> Result<Score, PhysicsError> {
        let validated = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        // ARM SVE detection is typically handled at runtime via auxiliary vectors or platform-specific probes.
        // For now, we fall back to scalar or NEON if SVE implementation is not fully ready for the target.
        // This is a placeholder for the actual SVE kernel entry.
        score_layout_scalar(&self.ctx, &validated, scratch).map(Score::from_scaled_i64)
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let validated = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        let layout_slice = validated.as_slice();

        crate::kernel::compute::state::with_scratch(|s| {
            let key_count = self.ctx.key_count;
            let (starts, counts, indices, offsets, used, _char_usage, _flat_map, _, _) =
                s.get_mut_scratch();
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

            let mono = crate::kernel::compute::scoring::score_monograms(&eval_ctx)?.raw();
            let bigram = crate::kernel::compute::scoring::score_bigrams(&eval_ctx)?.raw();
            let trigram = crate::kernel::compute::scoring::score_trigrams(&eval_ctx)?.raw();
            s.clear_used();
            Ok((mono, bigram, trigram))
        })?
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _pos_map: &[keyforge_model::types::KeyIndex],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;

        crate::kernel::compute::state::with_scratch(|s| {
            let key_count = self.ctx.key_count;
            let (starts, counts, indices, offsets, used, _char_usage, _flat_map, _, _) =
                s.get_mut_scratch();
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
            )?;
            s.clear_used();
            Ok(delta)
        })?
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        crate::kernel::compute::analyze_layout(&self.ctx, &validated)
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
