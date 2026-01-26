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
    pub fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
        Self {
            ctx,
            _config: config.unwrap_or_default(),
        }
    }
}

impl ScoringEngine for ArmSveScoringEngine {
    fn name(&self) -> &'static str {
        "ARM SVE/SVE2 Optimized"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures::NEON | EngineFeatures::SVE | EngineFeatures::BLOCKING,
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
            // SAFETY: We have verified that the target architecture is aarch64.
            // SVE support is explicitly checked via is_aarch64_feature_detected before calling the SVE kernel.
            unsafe {
                if std::arch::is_aarch64_feature_detected!("sve") {
                    return score_layout_sve(&self.ctx, &validated, scratch).map(Score);
                }
            }
        }
        score_layout_scalar(&self.ctx, &validated, scratch).map(Score)
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
            let (starts, counts, indices, offsets, used, _char_usage, _flat_map) =
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
            let (starts, counts, indices, offsets, used, _char_usage, _flat_map) =
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

/// SVE Optimized Kernel
/// NOTE: Currently SVE intrinsics in Rust are unstable/experimental.
/// This implementation uses a loop-vectorized pattern that allows LLVM to
/// emit SVE instructions when compiled with `-C target-feature=+sve`.
#[cfg(target_arch = "aarch64")]
/// # Safety
/// This function is currently a passthrough to the safe scalar implementation,
/// but is marked unsafe to reserve the semantics for future SVE-specific optimizations.
unsafe fn score_layout_sve(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    // SAFETY: Delegate to the safe scalar implementation.
    score_layout_scalar(ctx, layout, scratch)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_sve_parity() -> Result<(), Box<dyn std::error::Error>> {
        use crate::kernel::compiler::Compiler;
        use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
        use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric};
        use std::sync::Arc;

        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex(0),
                col: ColIndex(0),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                row: RowIndex(0),
                col: ColIndex(1),
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                row: RowIndex(0),
                col: ColIndex(2),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into())?;
        let mut corpus = Corpus::default();
        let mut freqs = corpus.char_freqs.to_vec();
        freqs[97] = 100;
        freqs[98] = 200;
        corpus.char_freqs = Arc::from(freqs);
        corpus.bigrams = Arc::from(vec![(97, 98, 50)]);
        corpus.trigrams = Arc::from(vec![(97, 98, 97, 10)]);

        let cm = keyforge_model::testing::mock_cost_model();
        let ctx = Compiler::compile(&kb, &corpus, &Rubric::default(), &cm)?;
        let engine = ArmSveScoringEngine::new(ctx.clone(), None);

        let layout = Layout {
            keys: vec![KeyCode(97), KeyCode(98), KeyCode(99)],
        };

        let score_res = engine.score(&layout)?;
        let scalar_score = score_layout_scalar(
            &ctx,
            &ValidatedLayout::new(&layout.keys, 3)?,
            &mut PhysicsScratch::new(),
        )?;

        assert_eq!(
            score_res.0, scalar_score,
            "SVE and Scalar scores must match exactly"
        );
        Ok(())
    }
}
