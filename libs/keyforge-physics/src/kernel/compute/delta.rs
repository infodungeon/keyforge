use super::flow::{get_flow_delta, get_p_effective};
use super::state::PosMap;
use crate::kernel::{
    types::{Score, ValidatedLayout},
    EngineContext,
};

#[allow(
    clippy::similar_names,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]
pub(crate) fn calculate_swap_delta(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    pos_map: &PosMap<'_>,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let layout_slice = layout.as_slice();
    if idx_a >= layout_slice.len() || idx_b >= layout_slice.len() {
        return 0;
    }
    let code_a = layout_slice[idx_a];
    let code_b = layout_slice[idx_b];
    if code_a == code_b {
        return 0;
    }

    let mut delta = 0i64;

    // 1. Monograms
    let freq_a = ctx.char_freqs[code_a.0 as usize] as i64;
    let freq_b = ctx.char_freqs[code_b.0 as usize] as i64;

    let candidates_a = pos_map.get(code_a.0 as usize);
    let candidates_b = pos_map.get(code_b.0 as usize);

    // code_a delta
    let mut min_old_a = Score(i64::MAX);
    let mut min_new_a = Score(i64::MAX);
    for &p in candidates_a {
        let p_idx = p as usize;
        let c_old = ctx.key_costs[p_idx];
        if c_old < min_old_a {
            min_old_a = c_old;
        }

        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx.key_costs[p_new];
        if c_new < min_new_a {
            min_new_a = c_new;
        }
    }
    delta += (min_new_a.0 - min_old_a.0) * freq_a;

    // code_b delta
    let mut min_old_b = Score(i64::MAX);
    let mut min_new_b = Score(i64::MAX);
    for &p in candidates_b {
        let p_idx = p as usize;
        let c_old = ctx.key_costs[p_idx];
        if c_old < min_old_b {
            min_old_b = c_old;
        }

        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx.key_costs[p_new];
        if c_new < min_new_b {
            min_new_b = c_new;
        }
    }
    delta += (min_new_b.0 - min_old_b.0) * freq_b;

    // 2. Bigrams
    // Bigrams(a, x)
    let start_a = ctx.bigram_starts[code_a.0 as usize];
    let end_a = ctx.bigram_starts[code_a.0 as usize + 1];
    for k in start_a..end_a {
        let c2 = ctx.bigram_others[k];
        let candidates2 = pos_map.get(c2.0 as usize);
        if candidates2.is_empty() {
            continue;
        }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates_a {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates2 {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);

                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code_a.0, c2.0)) {
                    cost_old = cost_old + mod_val;
                    cost_new = cost_new + mod_val;
                }

                if cost_old < min_old {
                    min_old = cost_old;
                }
                if cost_new < min_new {
                    min_new = cost_new;
                }
            }
        }
        delta += (min_new.0 - min_old.0) * i64::from(ctx.bigram_freqs[k]);
    }

    // Bigrams(b, x)
    let start_b = ctx.bigram_starts[code_b.0 as usize];
    let end_b = ctx.bigram_starts[code_b.0 as usize + 1];
    for k in start_b..end_b {
        let c2 = ctx.bigram_others[k];
        let candidates2 = pos_map.get(c2.0 as usize);
        if candidates2.is_empty() {
            continue;
        }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates_b {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates2 {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);

                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code_b.0, c2.0)) {
                    cost_old = cost_old + mod_val;
                    cost_new = cost_new + mod_val;
                }

                if cost_old < min_old {
                    min_old = cost_old;
                }
                if cost_new < min_new {
                    min_new = cost_new;
                }
            }
        }
        delta += (min_new.0 - min_old.0) * i64::from(ctx.bigram_freqs[k]);
    }

    // Bigrams(x, a) where x != a, x != b
    let start_rev_a = ctx.bigram_rev_starts[code_a.0 as usize];
    let end_rev_a = ctx.bigram_rev_starts[code_a.0 as usize + 1];
    for k in start_rev_a..end_rev_a {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b {
            continue;
        }
        let candidates1 = pos_map.get(c1.0 as usize);
        if candidates1.is_empty() {
            continue;
        }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates1 {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates_a {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);

                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1.0, code_a.0)) {
                    cost_old = cost_old + mod_val;
                    cost_new = cost_new + mod_val;
                }

                if cost_old < min_old {
                    min_old = cost_old;
                }
                if cost_new < min_new {
                    min_new = cost_new;
                }
            }
        }
        delta += (min_new.0 - min_old.0) * i64::from(ctx.bigram_rev_freqs[k]);
    }

    // Bigrams(x, b) where x != a, x != b
    let start_rev_b = ctx.bigram_rev_starts[code_b.0 as usize];
    let end_rev_b = ctx.bigram_rev_starts[code_b.0 as usize + 1];
    for k in start_rev_b..end_rev_b {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b {
            continue;
        }
        let candidates1 = pos_map.get(c1.0 as usize);
        if candidates1.is_empty() {
            continue;
        }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates1 {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates_b {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);

                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1.0, code_b.0)) {
                    cost_old = cost_old + mod_val;
                    cost_new = cost_new + mod_val;
                }

                if cost_old < min_old {
                    min_old = cost_old;
                }
                if cost_new < min_new {
                    min_new = cost_new;
                }
            }
        }
        delta += (min_new.0 - min_old.0) * i64::from(ctx.bigram_rev_freqs[k]);
    }

    // 3. Trigrams (Incremental)
    if !ctx.trigram_freqs.is_empty() {
        let ca = code_a.0 as usize;
        let cb = code_b.0 as usize;

        // Starts(a)
        let s_a = ctx.trigram_starts[ca];
        let e_a = ctx.trigram_starts[ca + 1];
        for k in s_a..e_a {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let freq = i64::from(ctx.trigram_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, code_a, c2, c3, idx_a, idx_b) * freq;
        }

        // Starts(b)
        let s_b = ctx.trigram_starts[cb];
        let e_b = ctx.trigram_starts[cb + 1];
        for k in s_b..e_b {
            let c2 = ctx.trigram_others1[k];
            let c3 = ctx.trigram_others2[k];
            let freq = i64::from(ctx.trigram_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, code_b, c2, c3, idx_a, idx_b) * freq;
        }

        // Mid(a) where c1 != a and c1 != b
        let s_ma = ctx.trigram_mid_starts[ca];
        let e_ma = ctx.trigram_mid_starts[ca + 1];
        for k in s_ma..e_ma {
            let c1 = ctx.trigram_mid_others1[k];
            if c1 == code_a || c1 == code_b {
                continue;
            }
            let c3 = ctx.trigram_mid_others2[k];
            let freq = i64::from(ctx.trigram_mid_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, code_a, c3, idx_a, idx_b) * freq;
        }

        // Mid(b) where c1 != a and c1 != b
        let s_mb = ctx.trigram_mid_starts[cb];
        let e_mb = ctx.trigram_mid_starts[cb + 1];
        for k in s_mb..e_mb {
            let c1 = ctx.trigram_mid_others1[k];
            if c1 == code_a || c1 == code_b {
                continue;
            }
            let c3 = ctx.trigram_mid_others2[k];
            let freq = i64::from(ctx.trigram_mid_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, code_b, c3, idx_a, idx_b) * freq;
        }

        // Ends(a) where c1 != a, c1 != b, c2 != a, c2 != b
        let s_ea = ctx.trigram_end_starts[ca];
        let e_ea = ctx.trigram_end_starts[ca + 1];
        for k in s_ea..e_ea {
            let c1 = ctx.trigram_end_others1[k];
            let c2 = ctx.trigram_end_others2[k];
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b {
                continue;
            }
            let freq = i64::from(ctx.trigram_end_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_a, idx_a, idx_b) * freq;
        }

        // Ends(b) where c1 != a, c1 != b, c2 != a, c2 != b
        let s_eb = ctx.trigram_end_starts[cb];
        let e_eb = ctx.trigram_end_starts[cb + 1];
        for k in s_eb..e_eb {
            let c1 = ctx.trigram_end_others1[k];
            let c2 = ctx.trigram_end_others2[k];
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b {
                continue;
            }
            let freq = i64::from(ctx.trigram_end_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_b, idx_a, idx_b) * freq;
        }
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::compute::PhysicsScratch;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use proptest::prelude::*;
    use rand::SeedableRng;
    use std::sync::Arc;

    fn load_cost_model_fixture() -> CostModel {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/default_cost_model.json");
        let json = std::fs::read_to_string(path).expect("Failed to read fixture");
        serde_json::from_str(&json).expect("Failed to parse fixture")
    }

    fn kb_and_layout_strategy() -> impl Strategy<Value = (Keyboard, Vec<KeyCode>)> {
        (10..50usize).prop_flat_map(|count| {
            let kb_strat = prop::collection::vec(
                (
                    -20.0..20.0f32,
                    -20.0..20.0f32,
                    0u8..2,
                    0u8..5,
                    -5i8..5,
                    -10i8..15,
                ),
                count,
            )
            .prop_map(move |keys_data| {
                let keys = keys_data
                    .into_iter()
                    .enumerate()
                    .map(|(i, (x, y, hand, finger, row, col))| KeyNode {
                        index: i,
                        label: format!("k{}", i),
                        hand: HandIndex(hand),
                        finger: FingerIndex::new_unchecked(finger),
                        row: RowIndex(row),
                        col: ColIndex(col),
                        x,
                        y,
                        is_home: row == 1,
                        ..Default::default()
                    })
                    .collect();
                Keyboard::new(keys, 1).unwrap()
            });

            let layout_strat = prop::collection::vec(0u16..255, count)
                .prop_map(|codes| codes.into_iter().map(KeyCode).collect::<Vec<_>>());

            (kb_strat, layout_strat)
        })
    }

    fn corpus_strategy(char_range: std::ops::Range<u16>) -> impl Strategy<Value = Corpus> {
        (
            prop::collection::vec((char_range.clone(), char_range.clone(), 1u32..1000), 0..20),
            prop::collection::vec(
                (
                    char_range.clone(),
                    char_range.clone(),
                    char_range.clone(),
                    1u32..1000,
                ),
                0..20,
            ),
            prop::collection::vec(0u64..1000, 256),
        )
            .prop_map(|(bigrams, trigrams, char_freqs)| {
                let mut c = Corpus::default();
                c.bigrams = bigrams;
                c.trigrams = trigrams;
                c.char_freqs = char_freqs;
                c
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_delta_validity(
            (kb, mut layout_keys) in kb_and_layout_strategy(),
            cp in corpus_strategy(0..30),
            seed in any::<u64>(),
            swap_idx_1 in 0..100usize,
            swap_idx_2 in 0..100usize
        ) {
            let len = layout_keys.len();
            if len < 2 { return Ok(()); }

            let i = swap_idx_1 % len;
            let j = swap_idx_2 % len;
            if i == j { return Ok(()); }

            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            use rand::seq::SliceRandom;
            layout_keys.shuffle(&mut rng);

            let rubric = Rubric::default();
            let cost_model = load_cost_model_fixture();
            let engine = crate::EngineFactory::new_generic(&Arc::new(kb), &Arc::new(cp), &Arc::new(rubric), &cost_model).unwrap();

            let layout_for_score = Layout::new_unchecked(layout_keys.clone());
            let score_before = engine.score(&layout_for_score).unwrap().0;
            if score_before == i64::MAX { return Ok(()); }

            let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
            let mut scratch = PhysicsScratch::new();
            let pm = PosMap::from_scratch(
                &layout_keys,
                engine.key_count(),
                scratch.starts.as_mut_slice(),
                scratch.counts.as_mut_slice(),
                scratch.indices.as_mut_slice(),
                scratch.current_offsets.as_mut_slice(),
                &mut scratch.used_keys,
            );

            let delta = calculate_swap_delta(engine.context(), &validated, &pm, i, j);

            layout_keys.swap(i, j);
            let swapped_layout = Layout::new_unchecked(layout_keys.clone());
            let score_after = engine.score(&swapped_layout).unwrap().0;
            let actual_delta = score_after - score_before;

            prop_assert_eq!(
                actual_delta, delta,
                "Delta mismatch! Actual: {}, Calculated: {}",
                actual_delta, delta
            );
        }
    }

    #[test]
    fn test_calculate_swap_delta_with_modifiers() {
        let keys: Vec<KeyNode> = (0..2).map(|i| KeyNode {
             index: i,
             hand: HandIndex(0),
             finger: FingerIndex::new_unchecked(i as u8),
             row: RowIndex(0),
             col: ColIndex(i as i8),
             ..Default::default()
        }).collect();
        let kb = Keyboard::new(keys, 1).unwrap();
        
        let mut cp = Corpus::default();
        cp.char_freqs = vec![0; 256];
        cp.char_freqs[97] = 100; // 'a'
        cp.char_freqs[98] = 100; // 'b'
        cp.bigrams = vec![(97, 98, 100)];

        let rubric = Rubric::default();
        let cost_model = load_cost_model_fixture();
        
        let engine = crate::EngineFactory::new_generic(
            &Arc::new(kb), 
            &Arc::new(cp), 
            &Arc::new(rubric), 
            &cost_model
        ).unwrap();
        
        let mut ctx = engine.context().clone();

        let layout_keys = vec![KeyCode(97), KeyCode(98)];
        let validated = ValidatedLayout::new(&layout_keys, engine.key_count()).unwrap();
        
        let mut scratch = PhysicsScratch::new();
        let pos_map = PosMap::from_scratch(
            &layout_keys,
            engine.key_count(),
            scratch.starts.as_mut_slice(),
            scratch.counts.as_mut_slice(),
            scratch.indices.as_mut_slice(),
            scratch.current_offsets.as_mut_slice(),
            &mut scratch.used_keys,
        );

        // Force asymmetry in cost matrix so delta is non-zero
        // 0->1 (index 1) vs 1->0 (index 2)
        if ctx.cost_matrix.len() >= 4 {
             ctx.cost_matrix[1] = Score(10);
             ctx.cost_matrix[2] = Score(50);
        }

        ctx.sequence_modifiers.insert((97, 98), Score(100));
        
        let delta = calculate_swap_delta(&ctx, &validated, &pos_map, 0, 1);
        assert!(delta != 0);
    }
}
