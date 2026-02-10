// libs/keyforge-physics/src/engines/intel_comet_lake.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::config::EngineConfig;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone)]
pub(crate) struct CometLakeScoringEngine {
    ctx: EngineContext,
    config: EngineConfig,
}

impl CometLakeScoringEngine {
    #[must_use]
    pub(crate) fn new(ctx: EngineContext, config: Option<EngineConfig>) -> Self {
        Self {
            ctx,
            config: config.unwrap_or_default(),
        }
    }
}

impl ScoringEngine for CometLakeScoringEngine {
    fn name(&self) -> &'static str {
        "Intel Comet Lake (AVX2) Optimized"
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
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return score_layout_avx2(&self.ctx, &v, scratch, &self.config)
                    .map(Score::from_scaled_i64);
            }
        }
        score_layout_scalar(&self.ctx, &v, scratch).map(Score::from_scaled_i64)
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn score_layout_avx2(
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
        score_simple_avx2(&e_ctx, flat_map)?
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
    flat_map: &[keyforge_model::types::KeyIndex],
) -> Result<i64, PhysicsError> {
    let s1 = score_monograms_avx2(ctx, flat_map)?;
    let s2 = score_bigrams_avx2(ctx, flat_map, s1)?;
    score_trigrams_avx2(ctx, flat_map, s2)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_monograms_avx2(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[keyforge_model::types::KeyIndex],
) -> Result<i64, PhysicsError> {
    let mut total_score = 0i64;
    for &code in ctx.pos_map.used_keys() {
        let freq = ctx.engine.corpus.char_freqs[code.as_usize()];
        let p = flat_map[code.as_usize()];
        let cost = ctx.engine.geometry.key_costs[p.as_usize()];
        let f_i64 = i64::try_from(freq).unwrap_or(i64::MAX);
        total_score = total_score
            .checked_add(cost.raw().checked_mul(f_i64).ok_or_else(|| {
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
    flat_map: &[keyforge_model::types::KeyIndex],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    let key_count = ctx.engine.key_count;
    for &code1 in ctx.pos_map.used_keys() {
        let p1 = flat_map[code1.as_usize()].as_usize();
        let start = ctx.engine.corpus.bigram_starts[code1.as_usize()];
        let end = ctx.engine.corpus.bigram_starts[code1.as_usize() + 1];
        let others_ptr = ctx.engine.corpus.bigram_others.as_ptr();
        let freqs_ptr = ctx.engine.corpus.bigram_freqs.as_ptr();
        let mut k = start;
        while k < end {
            let p2 = unsafe { flat_map[others_ptr.add(k).read().as_usize()] };
            if p2.as_usize() < key_count {
                let freq_i128 = i128::from(unsafe { freqs_ptr.add(k).read() });
                let cost_i128 = i128::from(
                    ctx.engine.geometry.cost_matrix[p1 * key_count + p2.as_usize()].raw(),
                );
                let contribution = i64::try_from(
                    (freq_i128 * cost_i128).clamp(i128::from(i64::MIN), i128::from(i64::MAX)),
                )
                .unwrap_or(0);

                total_score = total_score.checked_add(contribution).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "AVX2 Bigram".into(),
                    }
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
    flat_map: &[keyforge_model::types::KeyIndex],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    let key_count_usize = ctx.engine.key_count;
    let mut pos_types = [0u8; 256];
    for (i, p_type) in pos_types.iter_mut().enumerate().take(key_count_usize) {
        *p_type = ctx.engine.geometry.hands[i].as_u8() * 5 + ctx.engine.geometry.fingers[i].as_u8();
    }
    let flow_table = build_flow_table_avx2(ctx);
    let mut type_map = vec![255u8; 65536].into_boxed_slice();
    for &code in ctx.pos_map.used_keys() {
        let p = flat_map[code.as_usize()];
        if p.as_usize() < key_count_usize {
            type_map[code.as_usize()] = pos_types[p.as_usize()];
        }
    }
    for &code1 in ctx.pos_map.used_keys() {
        let t1 = type_map[code1.as_usize()];
        if t1 == 255 {
            continue;
        }
        let (start, end, t1_off) = (
            ctx.engine.corpus.trigram_starts[code1.as_usize()],
            ctx.engine.corpus.trigram_starts[code1.as_usize() + 1],
            usize::from(t1) * 100,
        );
        let (o1_ptr, o2_ptr, f_ptr) = (
            ctx.engine.corpus.trigram_others1.as_ptr(),
            ctx.engine.corpus.trigram_others2.as_ptr(),
            ctx.engine.corpus.trigram_freqs.as_ptr(),
        );
        for ki in start..end {
            let (t2, t3) = unsafe {
                (
                    type_map[o1_ptr.add(ki).read().as_usize()],
                    type_map[o2_ptr.add(ki).read().as_usize()],
                )
            };
            if t2 != 255 && t3 != 255 {
                total_score = total_score
                    .checked_add(
                        i64::from(unsafe { f_ptr.add(ki).read() })
                            * flow_table[t1_off + usize::from(t2) * 10 + usize::from(t3)],
                    )
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "AVX2 Tri".into(),
                    })?;
            }
        }
    }
    Ok(total_score)
}

fn build_flow_table_avx2(ctx: &crate::kernel::EvaluationContext<'_>) -> Box<[i64]> {
    let mut flow_table = vec![0i64; 1000].into_boxed_slice();
    for t1 in 0u8..10 {
        for t2 in 0u8..10 {
            for t3 in 0u8..10 {
                let (t1_u, t2_u, t3_u) = (t1, t2, t3);
                flow_table[usize::from(t1) * 100 + usize::from(t2) * 10 + usize::from(t3)] =
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
                    .raw();
            }
        }
    }
    flow_table
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    #[test]
    fn test_comet_lake_parity() -> anyhow::Result<()> {
        use crate::kernel::compiler::Compiler;
        use keyforge_model::types::{
            ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex,
        };
        use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric};
        use std::sync::Arc;
        let keys = vec![
            KeyNode {
                index: KeyIndex::new(0),
                hand: HandIndex::LEFT,
                finger: FingerIndex::INDEX,
                row: RowIndex::new(0),
                col: ColIndex::new(0),
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex::new(1),
                hand: HandIndex::LEFT,
                finger: FingerIndex::MIDDLE,
                row: RowIndex::new(0),
                col: ColIndex::new(1),
                ..Default::default()
            },
            KeyNode {
                index: KeyIndex::new(2),
                hand: HandIndex::LEFT,
                finger: FingerIndex::RING,
                row: RowIndex::new(0),
                col: ColIndex::new(2),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, RowIndex::new(0), "test".into())?;
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
        let engine = CometLakeScoringEngine::new(ctx.clone(), None);
        let layout =
            Layout::new_unchecked(vec![KeyCode::new(97), KeyCode::new(98), KeyCode::new(99)]);
        assert_eq!(
            engine.score(&layout)?.raw(),
            score_layout_scalar(
                &ctx,
                &ValidatedLayout::new(layout.keys(), 3)?,
                &mut PhysicsScratch::try_new()?
            )?
        );
        Ok(())
    }
}
