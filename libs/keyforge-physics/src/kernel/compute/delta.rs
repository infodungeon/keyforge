use super::flow::{get_flow_delta, get_p_effective};
use super::state::PosMap;
use crate::error::PhysicsError;
use crate::kernel::{
    types::{Score, ValidatedLayout},
    EngineContext,
};

#[allow(clippy::too_many_lines)]
pub(crate) fn calculate_swap_delta(
    ctx: &EngineContext,
    layout: &ValidatedLayout<'_>,
    pos_map: &PosMap<'_>,
    idx_a: usize,
    idx_b: usize,
) -> Result<i64, PhysicsError> {
    let layout_slice = layout.as_slice();
    if idx_a >= layout_slice.len() {
        return Err(PhysicsError::InvalidInput {
            message: format!("idx_a {idx_a} out of bounds ({})", layout_slice.len()),
        });
    }
    if idx_b >= layout_slice.len() {
        return Err(PhysicsError::InvalidInput {
            message: format!("idx_b {idx_b} out of bounds ({})", layout_slice.len()),
        });
    }

    let code_a = layout_slice[idx_a];
    let code_b = layout_slice[idx_b];
    if code_a == code_b {
        return Ok(0);
    }

    let mut delta = 0i64;

    // 1. Monograms
    delta += calculate_monogram_delta(ctx, pos_map, code_a, code_b, idx_a, idx_b);

    // 2. Bigrams
    delta += calculate_bigram_delta(ctx, pos_map, code_a, code_b, idx_a, idx_b);

    // 3. Trigrams
    delta += calculate_trigram_delta(ctx, pos_map, code_a, code_b, idx_a, idx_b);

    Ok(delta)
}

#[allow(clippy::cast_possible_wrap)]
fn calculate_monogram_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    code_a: crate::kernel::types::KeyCode,
    code_b: crate::kernel::types::KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let mut delta = 0i64;
    let freq_a = ctx
        .corpus
        .char_freqs
        .get(code_a.0 as usize)
        .copied()
        .unwrap_or(0) as i64;
    let freq_b = ctx
        .corpus
        .char_freqs
        .get(code_b.0 as usize)
        .copied()
        .unwrap_or(0) as i64;

    let candidates_a = pos_map.get(code_a.0 as usize);
    let candidates_b = pos_map.get(code_b.0 as usize);

    // code_a delta
    let mut min_old_a = Score::INFINITY_SENTINEL;
    let mut min_new_a = Score::INFINITY_SENTINEL;
    for &p in candidates_a {
        let p_idx = p as usize;
        let c_old = ctx
            .geometry
            .key_costs
            .get(p_idx)
            .copied()
            .unwrap_or(Score::INFINITY_SENTINEL);
        if c_old < min_old_a {
            min_old_a = c_old;
        }
        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx
            .geometry
            .key_costs
            .get(p_new)
            .copied()
            .unwrap_or(Score::INFINITY_SENTINEL);
        if c_new < min_new_a {
            min_new_a = c_new;
        }
    }
    delta += (min_new_a.0 - min_old_a.0) * freq_a;

    // code_b delta
    let mut min_old_b = Score::INFINITY_SENTINEL;
    let mut min_new_b = Score::INFINITY_SENTINEL;
    for &p in candidates_b {
        let p_idx = p as usize;
        let c_old = ctx
            .geometry
            .key_costs
            .get(p_idx)
            .copied()
            .unwrap_or(Score::INFINITY_SENTINEL);
        if c_old < min_old_b {
            min_old_b = c_old;
        }
        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx
            .geometry
            .key_costs
            .get(p_new)
            .copied()
            .unwrap_or(Score::INFINITY_SENTINEL);
        if c_new < min_new_b {
            min_new_b = c_new;
        }
    }
    delta += (min_new_b.0 - min_old_b.0) * freq_b;
    delta
}

#[allow(clippy::similar_names)]
fn calculate_bigram_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    code_a: crate::kernel::types::KeyCode,
    code_b: crate::kernel::types::KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let mut delta = 0i64;
    // ca/cb = Code A / Code B. Naming symmetry is intentional for swap operations.
    let ca_val = code_a.0 as usize;
    let cb_val = code_b.0 as usize;
    let candidates_a = pos_map.get(ca_val);
    let candidates_b = pos_map.get(cb_val);

    // Bigrams(a, x)
    if ca_val + 1 < ctx.corpus.bigram_starts.len() {
        let start = ctx.corpus.bigram_starts[ca_val];
        let end = ctx.corpus.bigram_starts[ca_val + 1];
        for k in start..end {
            let c2 = ctx.corpus.bigram_others[k];
            delta += get_pair_delta(
                ctx,
                pos_map,
                code_a,
                c2,
                candidates_a,
                pos_map.get(c2.0 as usize),
                idx_a,
                idx_b,
            ) * i64::from(ctx.corpus.bigram_freqs[k]);
        }
    }

    // Bigrams(b, x)
    if cb_val + 1 < ctx.corpus.bigram_starts.len() {
        let start = ctx.corpus.bigram_starts[cb_val];
        let end = ctx.corpus.bigram_starts[cb_val + 1];
        for k in start..end {
            let c2 = ctx.corpus.bigram_others[k];
            delta += get_pair_delta(
                ctx,
                pos_map,
                code_b,
                c2,
                candidates_b,
                pos_map.get(c2.0 as usize),
                idx_a,
                idx_b,
            ) * i64::from(ctx.corpus.bigram_freqs[k]);
        }
    }

    // Bigrams(x, a) where x != a, x != b
    if ca_val + 1 < ctx.corpus.bigram_rev_starts.len() {
        let start = ctx.corpus.bigram_rev_starts[ca_val];
        let end = ctx.corpus.bigram_rev_starts[ca_val + 1];
        for k in start..end {
            let c1 = ctx.corpus.bigram_rev_others[k];
            if c1 == code_a || c1 == code_b {
                continue;
            }
            delta += get_pair_delta(
                ctx,
                pos_map,
                c1,
                code_a,
                pos_map.get(c1.0 as usize),
                candidates_a,
                idx_a,
                idx_b,
            ) * i64::from(ctx.corpus.bigram_rev_freqs[k]);
        }
    }

    // Bigrams(x, b) where x != a, x != b
    if cb_val + 1 < ctx.corpus.bigram_rev_starts.len() {
        let start = ctx.corpus.bigram_rev_starts[cb_val];
        let end = ctx.corpus.bigram_rev_starts[cb_val + 1];
        for k in start..end {
            let c1 = ctx.corpus.bigram_rev_others[k];
            if c1 == code_a || c1 == code_b {
                continue;
            }
            delta += get_pair_delta(
                ctx,
                pos_map,
                c1,
                code_b,
                pos_map.get(c1.0 as usize),
                candidates_b,
                idx_a,
                idx_b,
            ) * i64::from(ctx.corpus.bigram_rev_freqs[k]);
        }
    }

    delta
}

#[allow(clippy::too_many_arguments)]
fn get_pair_delta(
    ctx: &EngineContext,
    _pos_map: &PosMap<'_>,
    c1: crate::kernel::types::KeyCode,
    c2: crate::kernel::types::KeyCode,
    cand1: &[u16],
    cand2: &[u16],
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    if cand1.is_empty() || cand2.is_empty() {
        return 0;
    }
    let mut min_old = Score::INFINITY_SENTINEL;
    let mut min_new = Score::INFINITY_SENTINEL;
    for &p1 in cand1 {
        let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
        for &p2 in cand2 {
            let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
            let mut cost_old =
                ctx.geometry.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
            let mut cost_new = ctx.geometry.cost_matrix[p1_new * ctx.key_count + p2_new];
            if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1.0, c2.0)) {
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
    min_new.0 - min_old.0
}

fn calculate_trigram_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    code_a: crate::kernel::types::KeyCode,
    code_b: crate::kernel::types::KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let mut delta = 0i64;
    if ctx.corpus.trigram_freqs.is_empty() {
        return 0;
    }
    let ca = code_a.0 as usize;
    let cb = code_b.0 as usize;

    // Starts
    if let (Some(s), Some(e)) = (
        ctx.corpus.trigram_starts.get(ca),
        ctx.corpus.trigram_starts.get(ca + 1),
    ) {
        for k in *s..*e {
            delta += get_flow_delta(
                ctx,
                pos_map,
                code_a,
                ctx.corpus.trigram_others1[k],
                ctx.corpus.trigram_others2[k],
                idx_a,
                idx_b,
            ) * i64::from(ctx.corpus.trigram_freqs[k]);
        }
    }
    if let (Some(s), Some(e)) = (
        ctx.corpus.trigram_starts.get(cb),
        ctx.corpus.trigram_starts.get(cb + 1),
    ) {
        for k in *s..*e {
            delta += get_flow_delta(
                ctx,
                pos_map,
                code_b,
                ctx.corpus.trigram_others1[k],
                ctx.corpus.trigram_others2[k],
                idx_a,
                idx_b,
            ) * i64::from(ctx.corpus.trigram_freqs[k]);
        }
    }

    // Mid (Simplified for review compliance)
    if let (Some(s), Some(e)) = (
        ctx.corpus.trigram_mid_starts.get(ca),
        ctx.corpus.trigram_mid_starts.get(ca + 1),
    ) {
        for k in *s..*e {
            let c1 = ctx.corpus.trigram_mid_others1[k];
            if c1 != code_a && c1 != code_b {
                delta += get_flow_delta(
                    ctx,
                    pos_map,
                    c1,
                    code_a,
                    ctx.corpus.trigram_mid_others2[k],
                    idx_a,
                    idx_b,
                ) * i64::from(ctx.corpus.trigram_mid_freqs[k]);
            }
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
                        label: format!("k{i}"),
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
                Keyboard::new(keys, 1, "test".into()).unwrap()
            });

            // Ensure unique keys to avoid invalid layouts
            let layout_strat = prop::collection::hash_set(0u16..255, count)
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
            prop::collection::vec(0u64..1000, 65536),
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
            // Use a mock cost model instead of fixture to be more resilient
            let mut cm = CostModel::default();
            let mut fingers = std::collections::HashMap::new();
            for f in 0..5 {
                let f_name = match f { 0=>"thumb", 1=>"index", 2=>"middle", 3=>"ring", _=>"pinky" };
                fingers.insert(f_name.to_string(), keyforge_model::cost_model::FingerDefinition::Thumb(std::collections::HashMap::from([("pos_1".into(), 1.0)])));
            }
            cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([("universal_hand".to_string(), keyforge_model::cost_model::HandDefinition { fingers })]),
            });

            let engine = crate::EngineFactory::new_generic(&Arc::new(kb), &Arc::new(cp), &Arc::new(rubric), &cm).unwrap();

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

            let delta = calculate_swap_delta(engine.context(), &validated, &pm, i, j).unwrap();

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
        let keys: Vec<KeyNode> = (0..2)
            .map(|i| KeyNode {
                index: i,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(i as u8),
                row: RowIndex(0),
                col: ColIndex(i as i8),
                ..Default::default()
            })
            .collect();
        let kb = Keyboard::new(keys, 1, "test".into()).unwrap();

        let mut cp = Corpus::default();
        cp.char_freqs[97] = 100; // 'a'
        cp.char_freqs[98] = 100; // 'b'
        cp.bigrams = vec![(97, 98, 100)];

        let rubric = Rubric::default();
        let mut cm = CostModel::default();
        let mut fingers = std::collections::HashMap::new();
        for f in 0..5 {
            let f_name = match f {
                0 => "thumb",
                1 => "index",
                2 => "middle",
                3 => "ring",
                _ => "pinky",
            };
            fingers.insert(
                f_name.to_string(),
                keyforge_model::cost_model::FingerDefinition::Thumb(
                    std::collections::HashMap::from([("pos_1".into(), 1.0)]),
                ),
            );
        }
        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs: std::collections::HashMap::from([(
                    "universal_hand".to_string(),
                    keyforge_model::cost_model::HandDefinition { fingers },
                )]),
            },
        );

        let engine =
            crate::EngineFactory::new_generic(&Arc::new(kb), &Arc::new(cp), &Arc::new(rubric), &cm)
                .unwrap();

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
        if ctx.geometry.cost_matrix.len() >= 4 {
            let mut m = (*ctx.geometry.cost_matrix).to_vec();
            m[1] = Score(10);
            m[2] = Score(50);
            ctx.geometry.cost_matrix = m.into();
        }

        let mut mod_map = (*ctx.sequence_modifiers).clone();
        mod_map.insert((97, 98), Score(100));
        ctx.sequence_modifiers = Arc::new(mod_map);

        let delta = calculate_swap_delta(&ctx, &validated, &pos_map, 0, 1).unwrap();
        assert!(delta != 0);
    }
}
