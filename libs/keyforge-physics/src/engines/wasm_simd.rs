// libs/keyforge-physics/src/engines/wasm_simd.rs

#![allow(unsafe_code)]
use super::{EngineCapabilities, EngineFeatures, ScoringEngine};
use crate::kernel::compute::{PhysicsScratch, PosMap};
use crate::kernel::{types::ValidatedLayout, EngineContext};
use crate::PhysicsError;
use keyforge_model::{AnalysisReport, Layout, Score, SwapSuggestion};

#[derive(Debug, Clone, Default)]
pub struct WasmSimdConfig;

#[derive(Debug, Clone)]
pub(crate) struct WasmSimdScoringEngine {
    pub(crate) ctx: EngineContext,
    _config: WasmSimdConfig,
}

impl WasmSimdScoringEngine {
    #[must_use]
    pub fn new(ctx: EngineContext, config: Option<WasmSimdConfig>) -> Self {
        Self {
            ctx,
            _config: config.unwrap_or_default(),
        }
    }
}

impl ScoringEngine for WasmSimdScoringEngine {
    fn name(&self) -> &'static str {
        "WASM SIMD Optimized"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            is_exact: false,
            features: EngineFeatures::WASM_SIMD,
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
        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                return score_layout_wasm(&self.ctx, &validated, scratch).map(Score);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
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

#[allow(dead_code)]
fn score_layout_scalar(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
    crate::kernel::compute::score_layout(ctx, layout, scratch)
}

#[cfg(target_arch = "wasm32")]
unsafe fn score_layout_wasm(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    scratch: &mut PhysicsScratch,
) -> Result<i64, PhysicsError> {
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
        let c = code as usize;
        let candidates = pm.get(c);
        if !candidates.is_empty() {
            flat_map[c] = candidates[0];
        }
    }

    let eval_ctx = crate::kernel::EvaluationContext {
        engine: ctx,
        pos_map: &pm,
    };

    let is_simple = pm
        .used_keys()
        .iter()
        .all(|&code| pm.get(code as usize).len() == 1)
        && ctx.sequence_modifiers.is_empty();

    let total = if is_simple {
        score_simple_wasm(&eval_ctx, flat_map)?
    } else {
        crate::kernel::compute::scoring::score_layout(ctx, layout, scratch)?
    };

    scratch.clear_used();
    Ok(total)
}

#[cfg(target_arch = "wasm32")]
unsafe fn score_simple_wasm(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[u16],
) -> Result<i64, PhysicsError> {
    use std::arch::wasm32::*;

    let mut total_score = 0i64;

    // 1. Monograms
    for &code in ctx.pos_map.used_keys() {
        let freq = ctx.engine.corpus.char_freqs[code as usize];
        let p = flat_map[code as usize];
        let cost = ctx.engine.geometry.key_costs[p as usize];
        total_score = total_score
            .checked_add(cost.0.checked_mul(freq as i64).ok_or_else(|| {
                PhysicsError::ScoreOverflow {
                    context: "WASM Monogram multiply".to_string(),
                }
            })?)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "WASM Monogram accumulation".to_string(),
            })?;
    }

    // 2. Bigrams
    for &code1 in ctx.pos_map.used_keys() {
        let c1_val = code1 as usize;
        let p1 = flat_map[c1_val] as usize;
        let start = ctx.engine.corpus.bigram_starts[c1_val];
        let end = ctx.engine.corpus.bigram_starts[c1_val + 1];

        let mut row_sum_v = i64x2_splat(0);
        let key_count = ctx.engine.key_count;
        let p1_offset = p1 * key_count;

        let costs_ptr = ctx.engine.geometry.cost_matrix.as_ptr();
        let freqs_ptr = ctx.engine.corpus.bigram_freqs.as_ptr();
        let others_ptr = ctx.engine.corpus.bigram_others.as_ptr();

        let mut k = start;
        while k + 2 <= end {
            // Load 2 KeyCodes (u16)
            let c2_0 = others_ptr.add(k).read().0 as usize;
            let c2_1 = others_ptr.add(k + 1).read().0 as usize;

            let p2_0 = flat_map[c2_0];
            let p2_1 = flat_map[c2_1];

            // Manual gather for costs
            let cost0 = if p2_0 < key_count as u16 {
                costs_ptr.add(p1_offset + (p2_0 as usize)).read().0
            } else {
                0
            };
            let cost1 = if p2_1 < key_count as u16 {
                costs_ptr.add(p1_offset + (p2_1 as usize)).read().0
            } else {
                0
            };

            // Load 2 frequencies (u32 -> i64)
            let freq0 = freqs_ptr.add(k).read() as i64;
            let freq1 = freqs_ptr.add(k + 1).read() as i64;

            // Multiply and accumulate
            // WASM doesn't have i64x2_mul, so we must mul individual lanes or use a different strategy
            // But wait, it HAS i64x2_mul in some proposals, but maybe not standard yet.
            // Let's check std::arch::wasm32
            // For now, let's use scalar mul and then pack if i64x2_mul is missing.
            let res0 = cost0 * freq0;
            let res1 = cost1 * freq1;
            row_sum_v = i64x2_add(row_sum_v, i64x2(res0, res1));

            k += 2;
        }

        // Horizontal sum of row_sum_v
        total_score = total_score
            .checked_add(i64x2_extract_lane::<0>(row_sum_v))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "WASM Bigram accumulation lane 0".to_string(),
            })?;
        total_score = total_score
            .checked_add(i64x2_extract_lane::<1>(row_sum_v))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "WASM Bigram accumulation lane 1".to_string(),
            })?;

        // Remainder
        while k < end {
            let c2 = others_ptr.add(k).read();
            let p2 = flat_map[c2.0 as usize];
            if p2 < key_count as u16 {
                let freq = freqs_ptr.add(k).read() as i64;
                let cost = ctx.engine.geometry.cost_matrix[p1 * key_count + p2 as usize].0;
                total_score = total_score.checked_add(cost * freq).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "WASM Bigram remainder accumulation".to_string(),
                    }
                })?;
            }
            k += 1;
        }
    }

    // 3. Trigrams
    total_score = score_trigrams_wasm(ctx, flat_map, total_score)?;

    Ok(total_score)
}

#[cfg(target_arch = "wasm32")]
unsafe fn score_trigrams_wasm(
    ctx: &crate::kernel::EvaluationContext<'_>,
    flat_map: &[u16],
    mut total_score: i64,
) -> Result<i64, PhysicsError> {
    use crate::kernel::mechanics::calculate_flow_cost;
    use std::arch::wasm32::*;

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
        let p = flat_map[code as usize];
        if p < key_count as u16 {
            type_map[code as usize] = pos_types[p as usize];
        }
    }

    for &code1 in ctx.pos_map.used_keys() {
        let t1 = type_map[code1 as usize];
        if t1 == 255 {
            continue;
        }

        let start = ctx.engine.corpus.trigram_starts[code1 as usize];
        let end = ctx.engine.corpus.trigram_starts[code1 as usize + 1];

        let others1_ptr = ctx.engine.corpus.trigram_others1.as_ptr();
        let others2_ptr = ctx.engine.corpus.trigram_others2.as_ptr();
        let freqs_ptr = ctx.engine.corpus.trigram_freqs.as_ptr();

        let t1_offset = (t1 as usize) * 100;
        let mut row_sum_v = i64x2_splat(0);

        let mut k = start;
        while k + 2 <= end {
            let c2_0 = others1_ptr.add(k).read().0 as usize;
            let c2_1 = others1_ptr.add(k + 1).read().0 as usize;

            let c3_0 = others2_ptr.add(k).read().0 as usize;
            let c3_1 = others2_ptr.add(k + 1).read().0 as usize;

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

            let freq0 = freqs_ptr.add(k).read() as i64;
            let freq1 = freqs_ptr.add(k + 1).read() as i64;

            row_sum_v = i64x2_add(row_sum_v, i64x2(cost0 * freq0, cost1 * freq1));

            k += 2;
        }

        total_score = total_score
            .checked_add(i64x2_extract_lane::<0>(row_sum_v))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "WASM Trigram accumulation lane 0".to_string(),
            })?;
        total_score = total_score
            .checked_add(i64x2_extract_lane::<1>(row_sum_v))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "WASM Trigram accumulation lane 1".to_string(),
            })?;

        while k < end {
            let c2 = others1_ptr.add(k).read();
            let c3 = others2_ptr.add(k).read();
            let t2 = type_map[c2.0 as usize];
            let t3 = type_map[c3.0 as usize];
            if t2 != 255 && t3 != 255 {
                let freq = freqs_ptr.add(k).read() as i64;
                let cost = flow_table[t1_offset + (t2 as usize) * 10 + (t3 as usize)];
                total_score = total_score.checked_add(cost * freq).ok_or_else(|| {
                    PhysicsError::ScoreOverflow {
                        context: "WASM Trigram remainder accumulation".to_string(),
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
    fn test_wasm_simd_parity() {
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
        let mut freqs = corpus.char_freqs.to_vec();
        freqs[97] = 100;
        freqs[98] = 200;
        corpus.char_freqs = Arc::from(freqs);
        corpus.bigrams = Arc::from(vec![(97, 98, 50)]);
        corpus.trigrams = Arc::from(vec![(97, 98, 97, 10)]);

        let cm = keyforge_model::testing::mock_cost_model();
        let ctx = Compiler::compile(&kb, &corpus, &Rubric::default(), &cm).unwrap();
        let engine = WasmSimdScoringEngine::new(ctx.clone(), None);

        let layout = Layout {
            keys: vec![KeyCode(97), KeyCode(98), KeyCode(99)],
        };

        let score_res = engine.score(&layout).unwrap();

        // Parity check (native only if wasm32, otherwise scalar path is taken anyway)
        let scalar_score = score_layout_scalar(
            &ctx,
            &ValidatedLayout::new(&layout.keys, 3).unwrap(),
            &mut PhysicsScratch::new(),
        )
        .unwrap();

        assert_eq!(
            score_res.0, scalar_score,
            "WASM and Scalar scores must match exactly"
        );
    }
}
