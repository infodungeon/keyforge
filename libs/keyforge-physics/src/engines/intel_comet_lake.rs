// libs/keyforge-physics/src/engines/intel_comet_lake.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::config::EngineConfig;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub(crate) struct IntelScoringEngine {
    pub(crate) ctx: EngineContext,
    config: EngineConfig,
}
impl IntelScoringEngine {
    #[must_use]
    pub fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
        Self {
            ctx,
            config: config.unwrap_or_default(),
        }
    }
}
impl ScoringEngine for IntelScoringEngine {
    fn name(&self) -> &'static str {
        "Intel Comet Lake (AVX2 Optimized)"
    }
    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures::AVX2 | EngineFeatures::BLOCKING,
        }
    }
    fn key_count(&self) -> usize {
        self.ctx.key_count
    }
    fn score(&self, layout: &Layout) -> Result<Score, PhysicsError> {
        std::thread_local! { static SCRATCH: std::cell::RefCell<PhysicsScratch> = std::cell::RefCell::new(PhysicsScratch::new()); }
        SCRATCH.with(|s| self.score_with_scratch(layout, &mut s.borrow_mut()))
    }
    fn score_with_scratch(
        &self,
        layout: &Layout,
        scratch: &mut PhysicsScratch,
    ) -> Result<Score, PhysicsError> {
        let v = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_x86_feature_detected!("avx2") {
            // SAFETY: We have explicitly verified the presence of AVX2 extensions via cpuid before execution.
            unsafe {
                return score_layout_avx2(&self.ctx, &v, scratch, &self.config).map(Score);
            }
        }
        score_layout_scalar(&self.ctx, &v, scratch).map(Score)
    }
    fn score_detailed(&self, layout: &Layout) -> Result<(i64, i64, i64), PhysicsError> {
        let v = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let mut s = PhysicsScratch::new();
        let (starts, counts, indices, offsets, used, _, _) = s.get_mut_scratch();
        let pm = PosMap::from_scratch(
            v.as_slice(),
            self.ctx.key_count,
            starts,
            counts,
            indices,
            offsets,
            used,
        );
        let ec = crate::kernel::EvaluationContext {
            engine: &self.ctx,
            pos_map: &pm,
        };
        Ok((
            crate::kernel::compute::scoring::score_monograms(&ec)?.0,
            crate::kernel::compute::scoring::score_bigrams(&ec)?.0,
            crate::kernel::compute::scoring::score_trigrams(&ec)?.0,
        ))
    }
    fn calculate_swap_delta(
        &self,
        layout: &Layout,
        _: &[u16],
        idx_a: usize,
        idx_b: usize,
    ) -> Result<i64, PhysicsError> {
        let v = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        let mut s = PhysicsScratch::new();
        let (starts, counts, indices, offsets, used, _, _) = s.get_mut_scratch();
        let pm = PosMap::from_scratch(
            v.as_slice(),
            self.ctx.key_count,
            starts,
            counts,
            indices,
            offsets,
            used,
        );
        crate::kernel::compute::calculate_swap_delta(&self.ctx, &v, &pm, idx_a, idx_b)
    }
    fn analyze(&self, layout: &Layout) -> Result<AnalysisReport, PhysicsError> {
        let v = ValidatedLayout::new(&layout.keys, self.ctx.key_count)?;
        Ok(crate::kernel::compute::analyze_layout(&self.ctx, &v))
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
unsafe fn score_layout_avx2(
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
        let cand = pm.get(code as usize);
        if !cand.is_empty() {
            flat_map[code as usize] = cand[0];
        }
    }
    let ec = crate::kernel::EvaluationContext {
        engine: ctx,
        pos_map: &pm,
    };
    let total = if pm
        .used_keys()
        .iter()
        .all(|&c| pm.get(c as usize).len() == 1)
        && ctx.sequence_modifiers.is_empty()
    {
        score_simple_avx2(&ec, flat_map)?
    } else {
        crate::kernel::compute::scoring::score_layout(ctx, layout, scratch)?
    };
    scratch.clear_used();
    Ok(total)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_simple_avx2(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[u16],
) -> Result<i64, PhysicsError> {
    let s1 = score_monograms_avx2(ctx, flat_map)?;
    let s2 = score_bigrams_avx2(ctx, flat_map, s1)?;
    score_trigrams_avx2(ctx, flat_map, s2)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_monograms_avx2(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[u16],
) -> Result<i64, PhysicsError> {
    let mut total_score = 0i64;
    for &code in ctx.pos_map.used_keys() {
        let freq = ctx.engine.corpus.char_freqs[code as usize];
        let p = flat_map[code as usize];
        let cost = ctx.engine.geometry.key_costs[p as usize];
        #[allow(clippy::cast_possible_wrap)]
        let f_i64 = freq as i64;
        total_score =
            total_score
                .checked_add(cost.0.checked_mul(f_i64).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "AVX2 Mono".into(),
                    }
                })?)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "AVX2 Mono Acc".into(),
                })?;
    }
    Ok(total_score)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_bigrams_avx2(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[u16],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    use std::arch::x86_64::{
        _mm256_add_epi64, _mm256_and_si256, _mm256_cvtepu32_epi64, _mm256_i32gather_epi64,
        _mm256_loadu_si256, _mm256_setzero_si256, _mm256_storeu_si256, _mm_loadu_si128,
    };
    let key_count = u16::try_from(ctx.engine.key_count).unwrap_or(u16::MAX);
    let key_count_usize = ctx.engine.key_count;
    for &code1 in ctx.pos_map.used_keys() {
        let p1 = flat_map[code1 as usize] as usize;
        let start = ctx.engine.corpus.bigram_starts[code1 as usize];
        let end = ctx.engine.corpus.bigram_starts[code1 as usize + 1];
        let mut row_sum = _mm256_setzero_si256();
        let costs_ptr = ctx
            .engine
            .geometry
            .cost_matrix
            .as_ptr()
            .add(p1 * key_count_usize);
        let (freqs_ptr, others_ptr) = (
            ctx.engine.corpus.bigram_freqs.as_ptr(),
            ctx.engine.corpus.bigram_others.as_ptr(),
        );
        let mut k = start;
        while k + 4 <= end {
            let mut idx = [0i32; 4];
            let mut msk = [0i64; 4];
            for i in 0..4 {
                // SAFETY: others_ptr is within corpus bounds [start, end). flat_map access is safe as others_ptr contains valid keycodes.
                let p2 = unsafe { flat_map[others_ptr.add(k + i).read().0 as usize] };
                if p2 < key_count {
                    msk[i] = -1;
                    idx[i] = i32::from(p2);
                }
            }
            // SAFETY: costs_ptr is valid for key_count elements in each row. idx contains valid indices.
            let cost_v = unsafe {
                _mm256_and_si256(
                    _mm256_i32gather_epi64(
                        costs_ptr.cast(),
                        _mm_loadu_si128(idx.as_ptr().cast()),
                        8,
                    ),
                    _mm256_loadu_si256(msk.as_ptr().cast()),
                )
            };
            // SAFETY: mul_epi64_avx2 correctly implements 64-bit multiplication for AVX2.
            row_sum = unsafe {
                _mm256_add_epi64(
                    row_sum,
                    mul_epi64_avx2(
                        cost_v,
                        _mm256_cvtepu32_epi64(_mm_loadu_si128(freqs_ptr.add(k).cast())),
                    ),
                )
            };
            k += 4;
        }
        let mut results = [0i64; 4];
        #[allow(clippy::cast_ptr_alignment)]
        // SAFETY: results is a local array of 4 i64s, matching the size of row_sum.
        unsafe {
            _mm256_storeu_si256(results.as_mut_ptr().cast(), row_sum);
        };
        for res in results {
            total_score =
                total_score
                    .checked_add(res)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX2 Bigram".into(),
                    })?;
        }
        while k < end {
            // SAFETY: Manual fallback loop for remaining bigrams. Bounds are guaranteed by 'end'.
            let p2 = unsafe { flat_map[others_ptr.add(k).read().0 as usize] };
            if p2 < key_count {
                total_score = total_score
                    .checked_add(
                        // SAFETY: others_ptr and freqs_ptr are valid at offset k.
                        i64::from(unsafe { freqs_ptr.add(k).read() })
                            * ctx.engine.geometry.cost_matrix[p1 * key_count_usize + p2 as usize].0,
                    )
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX2 Bigram rem".into(),
                    })?;
            }
            k += 1;
        }
    }
    Ok(total_score)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_trigrams_avx2(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[u16],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    use std::arch::x86_64::{
        _mm256_add_epi64, _mm256_and_si256, _mm256_cvtepu32_epi64, _mm256_i32gather_epi64,
        _mm256_loadu_si256, _mm256_setzero_si256, _mm256_storeu_si256, _mm_loadu_si128,
    };
    let flow_table = build_flow_table_avx2(ctx);
    let type_map = build_type_map_avx2(ctx, flat_map);

    for &code1 in ctx.pos_map.used_keys() {
        let t1 = type_map[code1 as usize];
        if t1 == 255 {
            continue;
        }
        let (start, end, t1_off) = (
            ctx.engine.corpus.trigram_starts[code1 as usize],
            ctx.engine.corpus.trigram_starts[code1 as usize + 1],
            (t1 as usize) * 100,
        );
        let mut row_sum = _mm256_setzero_si256();
        let (o1_ptr, o2_ptr, f_ptr) = (
            ctx.engine.corpus.trigram_others1.as_ptr(),
            ctx.engine.corpus.trigram_others2.as_ptr(),
            ctx.engine.corpus.trigram_freqs.as_ptr(),
        );
        let mut k = start;
        while k + 4 <= end {
            let mut idx = [0i32; 4];
            let mut msk = [0i64; 4];
            for i in 0..4 {
                let (t2, t3) = (
                    type_map[o1_ptr.add(k + i).read().0 as usize],
                    type_map[o2_ptr.add(k + i).read().0 as usize],
                );
                if t2 != 255 && t3 != 255 {
                    msk[i] = -1;
                    idx[i] = i32::from(t2) * 10 + i32::from(t3);
                }
            }
            let cost_v = _mm256_and_si256(
                _mm256_i32gather_epi64(
                    flow_table.as_ptr().add(t1_off).cast(),
                    _mm_loadu_si128(idx.as_ptr().cast()),
                    8,
                ),
                _mm256_loadu_si256(msk.as_ptr().cast()),
            );
            row_sum = _mm256_add_epi64(
                row_sum,
                mul_epi64_avx2(
                    cost_v,
                    _mm256_cvtepu32_epi64(_mm_loadu_si128(f_ptr.add(k).cast())),
                ),
            );
            k += 4;
        }
        let mut results = [0i64; 4];
        #[allow(clippy::cast_ptr_alignment)]
        _mm256_storeu_si256(results.as_mut_ptr().cast(), row_sum);
        for res in results {
            total_score =
                total_score
                    .checked_add(res)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX2 Tri".into(),
                    })?;
        }
        for ki in k..end {
            let (t2, t3) = (
                type_map[o1_ptr.add(ki).read().0 as usize],
                type_map[o2_ptr.add(ki).read().0 as usize],
            );
            if t2 != 255 && t3 != 255 {
                total_score = total_score
                    .checked_add(
                        i64::from(f_ptr.add(ki).read())
                            * flow_table[t1_off + (t2 as usize) * 10 + (t3 as usize)],
                    )
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX2 Tri rem".into(),
                    })?;
            }
        }
    }
    Ok(total_score)
}

fn build_type_map_avx2(ctx: &crate::kernel::EvaluationContext<'_>, flat_map: &[u16]) -> Box<[u8]> {
    let key_count_usize = ctx.engine.key_count;
    let mut pos_types = [0u8; 256];
    for (i, p_type) in pos_types.iter_mut().enumerate().take(key_count_usize) {
        *p_type = ctx.engine.geometry.hands[i].as_u8() * 5 + ctx.engine.geometry.fingers[i].as_u8();
    }
    let mut type_map = vec![255u8; 65536].into_boxed_slice();
    let limit = u16::try_from(key_count_usize).unwrap_or(u16::MAX);
    for &code in ctx.pos_map.used_keys() {
        let p = flat_map[code as usize];
        if p < limit {
            type_map[code as usize] = pos_types[p as usize];
        }
    }
    type_map
}

fn build_flow_table_avx2(ctx: &crate::kernel::EvaluationContext<'_>) -> Box<[i64]> {
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "avx2")]
/// Performs 64-bit multiplication using 32-bit AVX2 primitives.
///
/// # Safety
/// This function is unsafe because it uses SIMD intrinsics and must only be called
/// when AVX2 support is verified.
unsafe fn mul_epi64_avx2(
    a: std::arch::x86_64::__m256i,
    b: std::arch::x86_64::__m256i,
) -> std::arch::x86_64::__m256i {
    use std::arch::x86_64::{
        _mm256_add_epi64, _mm256_mul_epu32, _mm256_shuffle_epi32, _mm256_slli_epi64,
    };
    // SAFETY: Input parameters are valid SIMD registers.
    let (b_shuf, a_shuf) = (_mm256_shuffle_epi32(b, 0xF5), _mm256_shuffle_epi32(a, 0xF5));
    let (l_l_h, l_h_l, l_low_low) = (
        _mm256_mul_epu32(a, b_shuf),
        _mm256_mul_epu32(a_shuf, b),
        _mm256_mul_epu32(a, b),
    );
    _mm256_add_epi64(
        l_low_low,
        _mm256_slli_epi64(_mm256_add_epi64(l_l_h, l_h_l), 32),
    )
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    #[test]
    fn test_config_defaults() {
        let c = EngineConfig::default();
        assert!(c.use_prefetch);
        assert_eq!(c.l1d_size, 32 * 1024);
    }
    #[test]
    fn test_avx2_parity() {
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
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
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
        )
        .unwrap();
        let engine = IntelScoringEngine::new(ctx.clone(), None);
        let layout = Layout {
            keys: vec![KeyCode(97), KeyCode(98), KeyCode(99)],
        };
        assert_eq!(
            engine.score(&layout).unwrap().0,
            score_layout_scalar(
                &ctx,
                &ValidatedLayout::new(&layout.keys, 3).unwrap(),
                &mut PhysicsScratch::new()
            )
            .unwrap()
        );
    }
}
