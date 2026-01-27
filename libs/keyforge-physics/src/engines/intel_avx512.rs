// libs/keyforge-physics/src/engines/intel_avx512.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::config::EngineConfig;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub(crate) struct Avx512ScoringEngine {
    ctx: EngineContext,
    config: EngineConfig,
}

impl Avx512ScoringEngine {
    #[must_use]
    pub(crate) fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
        Self {
            ctx,
            config: config.unwrap_or_default(),
        }
    }
}

impl ScoringEngine for Avx512ScoringEngine {
    fn name(&self) -> &'static str {
        "Intel AVX-512 Optimized"
    }
    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures::AVX2 | EngineFeatures::AVX512 | EngineFeatures::BLOCKING,
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
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512dq")
            && is_x86_feature_detected!("avx512bw")
        {
            // SAFETY: We have explicitly verified the presence of AVX-512 foundation (F),
            // Doubleword/Quadword (DQ), and Byte/Word (BW) extensions via cpuid before execution.
            unsafe {
                return score_layout_avx512(&self.ctx, &validated, scratch, &self.config)
                    .map(Score);
            }
        }
        score_layout_scalar(&self.ctx, &validated, scratch).map(Score)
    }
    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let v = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;
        let layout_slice = v.as_slice();

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
        let validated = ValidatedLayout::new(layout.keys(), self.ctx.key_count)?;

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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn score_layout_avx512(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
    _: &EngineConfig,
) -> Result<i64, PhysicsError> {
    let (starts, counts, indices, offsets, used, _, flat_map) = scratch.get_mut_scratch();
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
            flat_map[code.0 as usize] = cand[0];
        }
    }
    let e_ctx = crate::kernel::EvaluationContext {
        engine: ctx,
        pos_map: &pm,
    };
    let is_simple =
        pm.used_keys().iter().all(|&c| pm.get(c).len() == 1) && ctx.sequence_modifiers.is_empty();
    let total = if is_simple {
        score_simple_avx512(&e_ctx, flat_map)?
    } else {
        crate::kernel::compute::scoring::score_layout(ctx, layout, scratch)?
    };
    scratch.clear_used();
    Ok(total)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f,avx512dq,avx512bw")]
unsafe fn score_simple_avx512(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
) -> Result<i64, PhysicsError> {
    let s1 = score_monograms_avx512(ctx, flat_map)?;
    let s2 = score_bigrams_avx512(ctx, flat_map, s1)?;
    score_trigrams_avx512(ctx, flat_map, s2)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f,avx512dq,avx512bw")]
unsafe fn score_monograms_avx512(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
) -> Result<i64, PhysicsError> {
    let mut total_score = 0i64;
    for &code in ctx.pos_map.used_keys() {
        let freq = ctx.engine.corpus.char_freqs[code.0 as usize];
        let p = flat_map[code.0 as usize];
        let cost = ctx.engine.geometry.key_costs[p.as_usize()];
        #[allow(clippy::cast_possible_wrap)]
        let f_i64 = freq as i64;
        total_score =
            total_score
                .checked_add(cost.0.checked_mul(f_i64).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "AVX-512 Mono".into(),
                    }
                })?)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "AVX-512 Mono Acc".into(),
                })?;
    }
    Ok(total_score)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f,avx512dq,avx512bw")]
unsafe fn score_bigrams_avx512(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    use std::arch::x86_64::{
        _mm256_loadu_si256, _mm512_add_epi64, _mm512_cvtepu32_epi64, _mm512_mask_i32gather_epi64,
        _mm512_mullo_epi64, _mm512_reduce_add_epi64, _mm512_setzero_si512,
    };
    let key_count = u16::try_from(ctx.engine.key_count).unwrap_or(u16::MAX);
    let key_count_usize = usize::from(key_count);
    for &code1 in ctx.pos_map.used_keys() {
        let p1 = flat_map[code1.0 as usize].as_usize();
        let start = ctx.engine.corpus.bigram_starts[code1.0 as usize];
        let end = ctx.engine.corpus.bigram_starts[code1.0 as usize + 1];
        let mut row_sum = _mm512_setzero_si512();
        let costs_ptr = ctx
            .engine
            .geometry
            .cost_matrix
            .as_ptr()
            .add(p1 * key_count_usize);
        let freqs_ptr = ctx.engine.corpus.bigram_freqs.as_ptr();
        let others_ptr = ctx.engine.corpus.bigram_others.as_ptr();
        let mut k = start;
        while k + 8 <= end {
            let mut mask = 0u8;
            let mut indices = [0i32; 8];
            for i in 0..8 {
                // SAFETY: others_ptr is derived from a ValidatedLayout, and k+i < end ensures we stay within bounds of the bigram data.
                let p2 = unsafe { flat_map[others_ptr.add(k + i).read().0 as usize] };
                if p2.as_usize() < key_count_usize {
                    mask |= 1 << i;
                    indices[i] = i32::from(p2.raw());
                }
            }
            let p2_v = _mm256_loadu_si256(indices.as_ptr().cast());
            // SAFETY: costs_ptr points to a row in the cost matrix. p2_v contains valid indices within that row.
            let cost_v = _mm512_mask_i32gather_epi64(
                _mm512_setzero_si512(),
                mask,
                p2_v,
                costs_ptr.cast(),
                8,
            );
            #[allow(clippy::cast_ptr_alignment)]
            // SAFETY: freqs_ptr is valid up to 'end'. We check k + 8 <= end.
            let freq_v = _mm512_cvtepu32_epi64(_mm256_loadu_si256(freqs_ptr.add(k).cast()));
            row_sum = _mm512_add_epi64(row_sum, _mm512_mullo_epi64(cost_v, freq_v));
            k += 8;
        }
        total_score = total_score
            .checked_add(_mm512_reduce_add_epi64(row_sum))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "AVX-512 Bigram".into(),
            })?;
        while k < end {
            // SAFETY: Manual fallback loop for remaining bigrams. Bounds are guaranteed by 'end'.
            let p2 = unsafe { flat_map[others_ptr.add(k).read().0 as usize] };
            if p2.as_usize() < key_count_usize {
                total_score = total_score
                    .checked_add(
                        // SAFETY: others_ptr and freqs_ptr are valid at offset k.
                        i64::from(unsafe { freqs_ptr.add(k).read() })
                            * ctx.engine.geometry.cost_matrix[p1 * key_count_usize + p2.as_usize()]
                                .0,
                    )
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX-512 Bigram rem".into(),
                    })?;
            }
            k += 1;
        }
    }
    Ok(total_score)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f,avx512dq,avx512bw")]
unsafe fn score_trigrams_avx512(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    use std::arch::x86_64::{
        _mm256_loadu_si256, _mm512_add_epi64, _mm512_cvtepu32_epi64, _mm512_mask_i32gather_epi64,
        _mm512_mullo_epi64, _mm512_reduce_add_epi64, _mm512_setzero_si512,
    };
    let key_count_usize = ctx.engine.key_count;
    let mut pos_types = [0u8; 256];
    for (i, p_type) in pos_types.iter_mut().enumerate().take(key_count_usize) {
        *p_type = ctx.engine.geometry.hands[i].as_u8() * 5 + ctx.engine.geometry.fingers[i].as_u8();
    }
    let flow_table = build_flow_table_avx512(ctx);
    let mut type_map = vec![255u8; 65536].into_boxed_slice();
    for &code in ctx.pos_map.used_keys() {
        let p = flat_map[code.0 as usize];
        if p.as_usize() < key_count_usize {
            type_map[code.0 as usize] = pos_types[p.as_usize()];
        }
    }
    for &code1 in ctx.pos_map.used_keys() {
        let t1 = type_map[code1.0 as usize];
        if t1 == 255 {
            continue;
        }
        let (start, end, t1_off) = (
            ctx.engine.corpus.trigram_starts[code1.0 as usize],
            ctx.engine.corpus.trigram_starts[code1.0 as usize + 1],
            (t1 as usize) * 100,
        );
        let mut row_sum = _mm512_setzero_si512();
        let (o1_ptr, o2_ptr, f_ptr) = (
            ctx.engine.corpus.trigram_others1.as_ptr(),
            ctx.engine.corpus.trigram_others2.as_ptr(),
            ctx.engine.corpus.trigram_freqs.as_ptr(),
        );
        let mut k = start;
        while k + 8 <= end {
            let mut idx = [0i32; 8];
            for i in 0..8 {
                // SAFETY: o1_ptr and o2_ptr are within corpus bounds [start, end).
                let (t2, t3) = unsafe {
                    (
                        type_map[o1_ptr.add(k + i).read().0 as usize],
                        type_map[o2_ptr.add(k + i).read().0 as usize],
                    )
                };
                idx[i] = if t2 != 255 && t3 != 255 {
                    i32::from(t2) * 10 + i32::from(t3)
                } else {
                    0
                };
            }
            // SAFETY: flow_table is indexed by (t1, t2, t3) where each is < 10. idx[i] < 100 and t1_off is t1 * 100.
            // Total index < 1000, which matches flow_table size.
            let cost_v = unsafe {
                _mm512_mask_i32gather_epi64(
                    _mm512_setzero_si512(),
                    0xFF,
                    _mm256_loadu_si256(idx.as_ptr().cast()),
                    flow_table.as_ptr().add(t1_off).cast(),
                    8,
                )
            };
            // SAFETY: f_ptr is valid up to 'end'. We check k + 8 <= end.
            let freq_v = _mm512_cvtepu32_epi64(_mm256_loadu_si256(f_ptr.add(k).cast()));
            row_sum = _mm512_add_epi64(row_sum, _mm512_mullo_epi64(cost_v, freq_v));
            k += 8;
        }
        total_score = total_score
            .checked_add(_mm512_reduce_add_epi64(row_sum))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "AVX-512 Tri".into(),
            })?;
        for ki in k..end {
            // SAFETY: ki < end ensures we are within corpus bounds.
            let (t2, t3) = unsafe {
                (
                    type_map[o1_ptr.add(ki).read().0 as usize],
                    type_map[o2_ptr.add(ki).read().0 as usize],
                )
            };
            if t2 != 255 && t3 != 255 {
                total_score = total_score
                    .checked_add(
                        // SAFETY: ki < end ensures f_ptr validity. Table lookup is within 1000 element bounds.
                        i64::from(unsafe { f_ptr.add(ki).read() })
                            * flow_table[t1_off + (t2 as usize) * 10 + (t3 as usize)],
                    )
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX-512 Tri rem".into(),
                    })?;
            }
        }
    }
    Ok(total_score)
}

fn build_flow_table_avx512(ctx: &crate::kernel::EvaluationContext<'_>) -> Box<[i64]> {
    let mut flow_table = vec![0i64; 1000].into_boxed_slice();
    for t1 in 0..10 {
        for t2 in 0..10 {
            for t3 in 0..10 {
                #[allow(clippy::cast_possible_truncation)]
                let (t1_u, t2_u, t3_u) = (t1 as u8, t2 as u8, t3 as u8);
                flow_table[t1 * 100 + t2 * 10 + t3] =
                    crate::kernel::mechanics::calculate_flow_cost(
                        keyforge_model::types::HandIndex::new(t1_u / 5),
                        keyforge_model::types::HandIndex::new(t2_u / 5),
                        keyforge_model::types::HandIndex::new(t3_u / 5),
                        keyforge_model::types::FingerIndex::new(t1_u % 5),
                        keyforge_model::types::FingerIndex::new(t2_u % 5),
                        keyforge_model::types::FingerIndex::new(t3_u % 5),
                        ctx.engine.penalty_redirect,
                        ctx.engine.bonus_roll,
                        ctx.engine.bonus_roll_out,
                    )
                    .0;
            }
        }
    }
    flow_table
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    #[test]
    fn test_avx512_parity() -> Result<(), Box<dyn std::error::Error>> {
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
        let kb = Keyboard::new(keys, RowIndex(0), "test".into())?;
        let mut corpus = Corpus::default();
        let mut f = corpus.char_freqs.to_vec();
        f[97] = 100;
        f[98] = 200;
        corpus.char_freqs = Arc::from(f);
        corpus.bigrams = Arc::from(vec![(97, 98, 50)]);
        corpus.trigrams = Arc::from(vec![(97, 98, 97, 10)]);
        let ctx = Compiler::compile(
            &kb,
            &corpus,
            &Rubric::default(),
            &keyforge_model::testing::mock_cost_model(),
        )?;
        let engine = Avx512ScoringEngine::new(ctx.clone(), None);
        let layout = Layout::new_unchecked(vec![KeyCode(97), KeyCode(98), KeyCode(99)]);
        assert_eq!(
            engine.score(&layout)?.0,
            score_layout_scalar(
                &ctx,
                &ValidatedLayout::new(layout.keys(), 3)?,
                &mut PhysicsScratch::try_new().unwrap()
            )?
        );
        Ok(())
    }
}
