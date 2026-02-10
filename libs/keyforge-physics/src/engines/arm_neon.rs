// libs/keyforge-physics/src/engines/arm_neon.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::config::EngineConfig;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

/// ARM NEON optimized physics engine implementation.
#[derive(Debug)]
pub struct ArmNeonScoringEngine {
    pub(crate) ctx: EngineContext,
    pub(crate) _config: EngineConfig,
}

impl ArmNeonScoringEngine {
    /// Creates a new `ArmNeonScoringEngine` instance.
    #[must_use]
    pub fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
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
            features: EngineFeatures::BLOCKING,
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
        let v = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                return score_layout_neon(&self.ctx, &v, scratch, &self.config)
                    .map(Score::from_scaled_i64);
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            score_layout_scalar(&self.ctx, &v, scratch).map(Score::from_scaled_i64)
        }
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let v = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        let layout_slice = v.as_slice();

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
        let v = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        crate::kernel::compute::analyze_layout(&self.ctx, &v)
    }

    fn suggest_improvements(&self, layout: &Layout, thumbs: bool) -> Vec<SwapSuggestion> {
        crate::analysis::heuristics::suggest_swaps(&self.ctx, layout, thumbs)
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

#[cfg(target_arch = "aarch64")]
unsafe fn score_layout_neon(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
    _: &EngineConfig,
) -> Result<i64, PhysicsError> {
    let (starts, counts, indices, offsets, used, _, flat_map, _, _) = scratch.get_mut_scratch();
    let pm = PosMap::from_scratch(
        layout.as_slice(),
        ctx.key_count,
        starts,
        counts,
        indices,
        offsets,
        used,
    );
    for &code in pm.used_keys() {
        let cand = pm.get(code);
        if !cand.is_empty() {
            flat_map[code.as_usize()] = cand[0];
        }
    }
    let e_ctx = crate::kernel::EvaluationContext {
        engine: ctx,
        pos_map: &pm,
    };
    let is_simple =
        pm.used_keys().iter().all(|&c| pm.get(c).len() == 1) && ctx.sequence_modifiers.is_empty();
    let total = if is_simple {
        score_simple_neon(&e_ctx, flat_map)?
    } else {
        crate::kernel::compute::scoring::score_layout(ctx, layout, scratch)?
    };
    scratch.clear_used();
    Ok(total)
}

#[cfg(target_arch = "aarch64")]
unsafe fn score_simple_neon(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
) -> Result<i64, PhysicsError> {
    use std::arch::aarch64::*;

    let mut total_score = 0i64;

    // 1. Monograms
    for &code in ctx.pos_map.used_keys() {
        let freq = ctx.engine.corpus.char_freqs[code.as_usize()];
        let p = flat_map[code.as_usize()];
        let cost = ctx.engine.geometry.key_costs[p.as_usize()];
        total_score = total_score
            .checked_add(cost.raw().checked_mul(i64::from(freq)).ok_or_else(|| {
                PhysicsError::ScoreOverflow {
                    context: "NEON Monogram multiply".to_string(),
                }
            })?)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "NEON Monogram accumulation".to_string(),
            })?;
    }

    // 2. Bigrams
    for &code1 in ctx.pos_map.used_keys() {
        let c1_val = code1.as_usize();
        let p1 = flat_map[c1_val].as_usize();
        let start = ctx.engine.corpus.bigram_starts[c1_val];
        let end = ctx.engine.corpus.bigram_starts[c1_val + 1];

        // SAFETY: vdupq_n_s64 is a safe NEON intrinsic when on aarch64.
        let mut row_sum_v = unsafe { vdupq_n_s64(0) };
        let key_count = ctx.engine.key_count;
        let p1_offset = p1 * key_count;

        let costs_ptr = ctx.engine.geometry.cost_matrix.as_ptr();
        let freqs_ptr = ctx.engine.corpus.bigram_freqs.as_ptr();
        let others_ptr = ctx.engine.corpus.bigram_others.as_ptr();

        let mut k = start;
        while k + 2 <= end {
            // Load 2 KeyCodes (u16)
            let c2_0 = unsafe { others_ptr.add(k).read().as_usize() };
            let c2_1 = unsafe { others_ptr.add(k + 1).read().as_usize() };

            let p2_0 = flat_map[c2_0].raw();
            let p2_1 = flat_map[c2_1].raw();

            // Manual gather for costs
            let key_count_u16 = u16::try_from(key_count).unwrap_or(u16::MAX);
            let cost0 = if p2_0 < key_count_u16 {
                unsafe { costs_ptr.add(p1_offset + usize::from(p2_0)).read().raw() }
            } else {
                0
            };
            let cost1 = if p2_1 < key_count_u16 {
                unsafe { costs_ptr.add(p1_offset + usize::from(p2_1)).read().raw() }
            } else {
                0
            };

            // SAFETY: vcombine_s64 and vcreate_s64 are safe NEON intrinsics on aarch64.
            let cost_v = unsafe { vcombine_s64(vcreate_s64(cost0), vcreate_s64(cost1)) };

            // Load 2 frequencies (u32 -> i64)
            let freq0 = i64::from(unsafe { freqs_ptr.add(k).read() });
            let freq1 = i64::from(unsafe { freqs_ptr.add(k + 1).read() });

            let freq_v = unsafe { vcombine_s64(vcreate_s64(freq0), vcreate_s64(freq1)) };

            // SAFETY: vgetq_lane_s64, vcreate_s64, vcombine_s64, and vaddq_s64 are safe NEON intrinsics on aarch64.
            let p0 = unsafe { vgetq_lane_s64(cost_v, 0) * vgetq_lane_s64(freq_v, 0) };
            let p1 = unsafe { vgetq_lane_s64(cost_v, 1) * vgetq_lane_s64(freq_v, 1) };

            let prod_v = unsafe { vcombine_s64(vcreate_s64(p0), vcreate_s64(p1)) };
            row_sum_v = unsafe { vaddq_s64(row_sum_v, prod_v) };

            k += 2;
        }

        // Horizontal sum of row_sum_v
        // SAFETY: vgetq_lane_s64 is safe on aarch64.
        total_score = total_score
            .checked_add(unsafe { vgetq_lane_s64(row_sum_v, 0) })
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "NEON Bigram accumulation lane 0".to_string(),
            })?;
        total_score = total_score
            .checked_add(unsafe { vgetq_lane_s64(row_sum_v, 1) })
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "NEON Bigram accumulation lane 1".to_string(),
            })?;

        // Handle remainder
        for i in k..end {
            let c2 = unsafe { others_ptr.add(i).read().as_usize() };
            let p2 = flat_map[c2].as_usize();
            let cost = unsafe { costs_ptr.add(p1_offset + p2).read().raw() };
            let freq = i64::from(unsafe { freqs_ptr.add(i).read() });
            total_score = total_score
                .checked_add(
                    cost.checked_mul(freq)
                        .ok_or_else(|| PhysicsError::ScoreOverflow {
                            context: "NEON Bigram remainder multiply".to_string(),
                        })?,
                )
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "NEON Bigram remainder accumulation".to_string(),
                })?;
        }
    }

    // 3. Trigrams
    for &code1 in ctx.pos_map.used_keys() {
        let c1_val = code1.as_usize();
        let h1 = u8::try_from(ctx.engine.geometry.hands[c1_val].as_usize()).unwrap_or(0);
        let f1 = u8::try_from(ctx.engine.geometry.fingers[c1_val].as_usize()).unwrap_or(0);

        let start = ctx.engine.corpus.trigram_starts[c1_val];
        let end = ctx.engine.corpus.trigram_starts[c1_val + 1];

        let others_ptr = ctx.engine.corpus.trigram_others.as_ptr();
        let freqs_ptr = ctx.engine.corpus.trigram_freqs.as_ptr();

        for k in start..end {
            let (c2_u16, c3_u16) = unsafe { others_ptr.add(k).read() };
            let c2 = usize::from(c2_u16);
            let c3 = usize::from(c3_u16);

            let h2 = u8::try_from(ctx.engine.geometry.hands[c2].as_usize()).unwrap_or(0);
            let f2 = u8::try_from(ctx.engine.geometry.fingers[c2].as_usize()).unwrap_or(0);
            let h3 = u8::try_from(ctx.engine.geometry.hands[c3].as_usize()).unwrap_or(0);
            let f3 = u8::try_from(ctx.engine.geometry.fingers[c3].as_usize()).unwrap_or(0);

            let t1 = crate::kernel::compute::trigram::classify_trigram(h1, f1, h2, f2, h3, f3);
            let freq = i64::from(unsafe { freqs_ptr.add(k).read() });
            let cost = ctx.engine.geometry.rubric.get_trigram_cost(t1).raw();

            total_score = total_score
                .checked_add(
                    cost.checked_mul(freq)
                        .ok_or_else(|| PhysicsError::ScoreOverflow {
                            context: "NEON Trigram multiply".to_string(),
                        })?,
                )
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "NEON Trigram accumulation".to_string(),
                })?;
        }
    }

    Ok(total_score)
}
