// libs/keyforge-physics/src/engines/arm_neon.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::config::EngineConfig;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub(crate) struct ArmNeonScoringEngine {
    pub(crate) ctx: EngineContext,
    _config: EngineConfig,
}

impl ArmNeonScoringEngine {
    #[must_use]
    pub(crate) fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
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
            features: EngineFeatures::NEON | EngineFeatures::BLOCKING,
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
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        #[cfg(target_arch = "aarch64")]
        {
            // SAFETY: We have verified that the target architecture is aarch64, which supports NEON instructions.
            unsafe {
                return score_layout_neon(&self.ctx, &validated, scratch).map(Score);
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            score_layout_scalar(&self.ctx, &validated, scratch).map(Score)
        }
    }

    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let layout_slice = validated.as_slice();

        crate::kernel::compute::state::with_scratch(|s| {
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
        })?
    }

    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _pos_map: &[keyforge_model::types::KeyIndex],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;

        crate::kernel::compute::state::with_scratch(|s| {
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
            )?;
            s.clear_used();
            Ok(delta)
        })?
    }

    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let validated = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
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

#[cfg(target_arch = "aarch64")]
unsafe fn score_layout_neon(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    use keyforge_model::types::KeyCode;
    let layout_slice = layout.as_slice();
    let (starts, counts, indices, offsets, used, _char_usage, flat_map) = scratch.get_mut_scratch();

    let pm = PosMap::from_scratch(
        layout_slice,
        ctx.key_count,
        starts,
        counts,
        indices,
        offsets,
        used,
    );

    // Populate flat_map for SIMD kernels (KeyCode -> KeyIndex)
    for &code in pm.used_keys() {
        let candidates = pm.get(code);
        if !candidates.is_empty() {
            flat_map[code.0 as usize] = candidates[0];
        }
    }

    let eval_ctx = crate::kernel::EvaluationContext {
        engine: ctx,
        pos_map: &pm,
    };

    let is_simple = pm.used_keys().iter().all(|&code| pm.get(code).len() == 1)
        && ctx.sequence_modifiers.is_empty();

    let total = if is_simple {
        // SAFETY: Evaluated only when is_simple is true, ensuring flat_map is fully populated and no sequence modifiers exist.
        unsafe { score_simple_neon(&eval_ctx, flat_map)? }
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
        let freq = ctx.engine.corpus.char_freqs[code.0 as usize];
        let p = flat_map[code.0 as usize];
        let cost = ctx.engine.geometry.key_costs[p.as_usize()];
        total_score = total_score
            .checked_add(cost.0.checked_mul(freq as i64).ok_or_else(|| {
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
        let c1_val = code1.0 as usize;
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
            // SAFETY: others_ptr is within corpus bounds [start, end).
            let c2_0 = unsafe { others_ptr.add(k).read().0 as usize };
            let c2_1 = unsafe { others_ptr.add(k + 1).read().0 as usize };

            let p2_0 = flat_map[c2_0].raw();
            let p2_1 = flat_map[c2_1].raw();

            // Manual gather for costs
            // SAFETY: costs_ptr is valid for key_count elements in each row.
            let cost0 = if p2_0 < key_count as u16 {
                unsafe { costs_ptr.add(p1_offset + (p2_0 as usize)).read().0 }
            } else {
                0
            };
            let cost1 = if p2_1 < key_count as u16 {
                unsafe { costs_ptr.add(p1_offset + (p2_1 as usize)).read().0 }
            } else {
                0
            };
            // SAFETY: vcombine_s64 and vcreate_s64 are safe NEON intrinsics on aarch64.
            let cost_v =
                unsafe { vcombine_s64(vcreate_s64(cost0 as u64), vcreate_s64(cost1 as u64)) };

            // Load 2 frequencies (u32 -> i64)
            // SAFETY: freqs_ptr is valid up to 'end'.
            let freq0 = unsafe { freqs_ptr.add(k).read() as i64 };
            let freq1 = unsafe { freqs_ptr.add(k + 1).read() as i64 };
            let freq_v =
                unsafe { vcombine_s64(vcreate_s64(freq0 as u64), vcreate_s64(freq1 as u64)) };

            // SAFETY: vgetq_lane_s64, vcreate_s64, vcombine_s64, and vaddq_s64 are safe NEON intrinsics on aarch64.
            let p0 = unsafe { vgetq_lane_s64(cost_v, 0) * vgetq_lane_s64(freq_v, 0) };
            let p1 = unsafe { vgetq_lane_s64(cost_v, 1) * vgetq_lane_s64(freq_v, 1) };
            let prod_v = unsafe { vcombine_s64(vcreate_s64(p0 as u64), vcreate_s64(p1 as u64)) };
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

        // Remainder
        while k < end {
            // SAFETY: others_ptr and freqs_ptr are valid at offset k < end.
            let c2 = unsafe { others_ptr.add(k).read() };
            let p2 = flat_map[c2.0 as usize];
            if p2.as_usize() < key_count {
                let freq = unsafe { freqs_ptr.add(k).read() as i64 };
                let cost = ctx.engine.geometry.cost_matrix[p1 * key_count + p2.as_usize()].0;
                total_score = total_score.checked_add(cost * freq).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "NEON Bigram remainder accumulation".to_string(),
                    }
                })?;
            }
            k += 1;
        }
    }

    // 3. Trigrams
    // SAFETY: score_trigrams_neon is documented below.
    total_score = unsafe { score_trigrams_neon(ctx, flat_map, total_score)? };

    Ok(total_score)
}

#[cfg(target_arch = "aarch64")]
unsafe fn score_trigrams_neon(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    use crate::kernel::mechanics::calculate_flow_cost;
    use std::arch::aarch64::*;

    let key_count = ctx.engine.key_count;
    let mut pos_types = [0u8; 256];
    for i in 0..key_count {
        let h = ctx.engine.geometry.hands[i].as_u8();
        let f = ctx.engine.geometry.fingers[i].as_u8();
        pos_types[i] = h * 5 + f;
    }

    let mut flow_table = [0i64; 1000];
    for t1 in 0..10 {
        for t2 in 0..10 {
            for t3 in 0..10 {
                let h1 = keyforge_model::types::HandIndex::new(t1 / 5);
                let h2 = keyforge_model::types::HandIndex::new(t2 / 5);
                let h3 = keyforge_model::types::HandIndex::new(t3 / 5);
                let f1 = keyforge_model::types::FingerIndex::new(t1 % 5);
                let f2 = keyforge_model::types::FingerIndex::new(t2 % 5);
                let f3 = keyforge_model::types::FingerIndex::new(t3 % 5);

                flow_table[(t1 as usize) * 100 + (t2 as usize) * 10 + (t3 as usize)] =
                    calculate_flow_cost(
                        h1,
                        h2,
                        h3,
                        f1,
                        f2,
                        f3,
                        ctx.engine.penalty_redirect,
                        ctx.engine.bonus_roll,
                        ctx.engine.bonus_roll_out,
                    )
                    .0;
            }
        }
    }

    let mut type_map = [255u8; 65536];
    for &code in ctx.pos_map.used_keys() {
        let p = flat_map[code.0 as usize];
        if p.as_usize() < key_count {
            type_map[code.0 as usize] = pos_types[p.as_usize()];
        }
    }

    for &code1 in ctx.pos_map.used_keys() {
        let t1 = type_map[code1.0 as usize];
        if t1 == 255 {
            continue;
        }

        let start = ctx.engine.corpus.trigram_starts[code1.0 as usize];
        let end = ctx.engine.corpus.trigram_starts[code1.0 as usize + 1];

        let others1_ptr = ctx.engine.corpus.trigram_others1.as_ptr();
        let others2_ptr = ctx.engine.corpus.trigram_others2.as_ptr();
        let freqs_ptr = ctx.engine.corpus.trigram_freqs.as_ptr();

        let t1_offset = (t1 as usize) * 100;
        // SAFETY: vdupq_n_s64 is safe on aarch64.
        let mut row_sum_v = unsafe { vdupq_n_s64(0) };

        let mut k = start;
        while k + 2 <= end {
            // SAFETY: pointers are within corpus bounds [start, end).
            let c2_0 = unsafe { others1_ptr.add(k).read().0 as usize };
            let c2_1 = unsafe { others1_ptr.add(k + 1).read().0 as usize };

            let c3_0 = unsafe { others2_ptr.add(k).read().0 as usize };
            let c3_1 = unsafe { others2_ptr.add(k + 1).read().0 as usize };

            let t2_0 = type_map[c2_0];
            let t2_1 = type_map[c2_1];
            let t3_0 = type_map[c3_0];
            let t3_1 = type_map[c3_1];

            let cost0 = if t2_0 != 255 && t3_0 != 255 {
                flow_table[t1_offset + (t2_0 as usize) * 10 + (t3_0 as usize)]
            } else {
                0
            };
            let cost1 = if t2_1 != 255 && t3_1 != 255 {
                flow_table[t1_offset + (t2_1 as usize) * 10 + (t3_1 as usize)]
            } else {
                0
            };
            // SAFETY: vcombine_s64 and vcreate_s64 are safe on aarch64.
            let cost_v =
                unsafe { vcombine_s64(vcreate_s64(cost0 as u64), vcreate_s64(cost1 as u64)) };

            // SAFETY: freqs_ptr is valid up to 'end'.
            let freq0 = unsafe { freqs_ptr.add(k).read() as i64 };
            let freq1 = unsafe { freqs_ptr.add(k + 1).read() as i64 };
            let freq_v =
                unsafe { vcombine_s64(vcreate_s64(freq0 as u64), vcreate_s64(freq1 as u64)) };

            // SAFETY: NEON intrinsics are safe on aarch64.
            let p0 = unsafe { vgetq_lane_s64(cost_v, 0) * vgetq_lane_s64(freq_v, 0) };
            let p1 = unsafe { vgetq_lane_s64(cost_v, 1) * vgetq_lane_s64(freq_v, 1) };
            let prod_v = unsafe { vcombine_s64(vcreate_s64(p0 as u64), vcreate_s64(p1 as u64)) };
            row_sum_v = unsafe { vaddq_s64(row_sum_v, prod_v) };

            k += 2;
        }

        // SAFETY: vgetq_lane_s64 is safe on aarch64.
        total_score = total_score
            .checked_add(unsafe { vgetq_lane_s64(row_sum_v, 0) })
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "NEON Trigram accumulation lane 0".to_string(),
            })?;
        total_score = total_score
            .checked_add(unsafe { vgetq_lane_s64(row_sum_v, 1) })
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "NEON Trigram accumulation lane 1".to_string(),
            })?;

        while k < end {
            // SAFETY: pointers are valid at offset k < end.
            let c2 = unsafe { others1_ptr.add(k).read() };
            let c3 = unsafe { others2_ptr.add(k).read() };
            let t2 = type_map[c2.0 as usize];
            let t3 = type_map[c3.0 as usize];
            if t2 != 255 && t3 != 255 {
                let freq = unsafe { freqs_ptr.add(k).read() as i64 };
                let cost = flow_table[t1_offset + (t2 as usize) * 10 + (t3 as usize)];
                total_score = total_score.checked_add(cost * freq).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "NEON Trigram remainder accumulation".to_string(),
                    }
                })?;
            }
            k += 1;
        }
    }

    Ok(total_score)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_neon_config_defaults() {
        let config = EngineConfig::default();
        assert_eq!(config.l1d_size, 32 * 1024);
    }

    #[test]
    fn test_neon_parity() {
        use crate::kernel::compiler::Compiler;
        use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
        use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Rubric};
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
        let kb = Keyboard::new(keys, RowIndex(0), "test".into()).unwrap();
        let mut corpus = Corpus::default();
        let mut freqs = corpus.char_freqs.to_vec();
        freqs[97] = 100;
        freqs[98] = 200;
        corpus.char_freqs = Arc::from(freqs);
        corpus.bigrams = Arc::from(vec![(97, 98, 50)]);
        corpus.trigrams = Arc::from(vec![(97, 98, 97, 10)]);

        let cm = keyforge_model::testing::mock_cost_model();
        let ctx = Compiler::compile(&kb, &corpus, &Rubric::default(), &cm).unwrap();
        let engine = ArmNeonScoringEngine::new(ctx.clone(), None);

        let layout = Layout {
            keys: vec![KeyCode(97), KeyCode(98), KeyCode(99)],
        };

        let score_res = engine.score(&layout).unwrap();

        // Parity check (native only if aarch64, otherwise scalar path is taken anyway)
        let scalar_score = score_layout_scalar(
            &ctx,
            &ValidatedLayout::new(&layout.keys, 3).unwrap(),
            &mut PhysicsScratch::try_new().unwrap(),
        )
        .unwrap();

        assert_eq!(
            score_res.0, scalar_score,
            "NEON and Scalar scores must match exactly"
        );
    }
}
