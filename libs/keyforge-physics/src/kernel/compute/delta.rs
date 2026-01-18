use crate::kernel::{EngineContext, types::{Score, ValidatedLayout}};
use super::state::PosMap;
use super::flow::{get_p_effective, get_flow_delta};

#[allow(clippy::similar_names, clippy::cast_possible_wrap, clippy::too_many_lines)]
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
        if c_old < min_old_a { min_old_a = c_old; }
        
        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx.key_costs[p_new];
        if c_new < min_new_a { min_new_a = c_new; }
    }
    delta += (min_new_a.0 - min_old_a.0) * freq_a;

    // code_b delta
    let mut min_old_b = Score(i64::MAX);
    let mut min_new_b = Score(i64::MAX);
    for &p in candidates_b {
        let p_idx = p as usize;
        let c_old = ctx.key_costs[p_idx];
        if c_old < min_old_b { min_old_b = c_old; }

        let p_new = get_p_effective(p_idx, idx_a, idx_b);
        let c_new = ctx.key_costs[p_new];
        if c_new < min_new_b { min_new_b = c_new; }
    }
    delta += (min_new_b.0 - min_old_b.0) * freq_b;

    // 2. Bigrams
    // Bigrams(a, x)
    let start_a = ctx.bigram_starts[code_a.0 as usize];
    let end_a = ctx.bigram_starts[code_a.0 as usize + 1];
    for k in start_a..end_a {
        let c2 = ctx.bigram_others[k];
        let candidates2 = pos_map.get(c2.0 as usize);
        if candidates2.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates_a {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates2 {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code_a.0, c2.0)) {
                    cost_old = cost_old.saturating_add(mod_val);
                    cost_new = cost_new.saturating_add(mod_val);
                }

                if cost_old < min_old { min_old = cost_old; }
                if cost_new < min_new { min_new = cost_new; }
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
        if candidates2.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates_b {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates2 {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(code_b.0, c2.0)) {
                    cost_old = cost_old.saturating_add(mod_val);
                    cost_new = cost_new.saturating_add(mod_val);
                }

                if cost_old < min_old { min_old = cost_old; }
                if cost_new < min_new { min_new = cost_new; }
            }
        }
        delta += (min_new.0 - min_old.0) * i64::from(ctx.bigram_freqs[k]);
    }

    // Bigrams(x, a) where x != a, x != b
    let start_rev_a = ctx.bigram_rev_starts[code_a.0 as usize];
    let end_rev_a = ctx.bigram_rev_starts[code_a.0 as usize + 1];
    for k in start_rev_a..end_rev_a {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b { continue; }
        let candidates1 = pos_map.get(c1.0 as usize);
        if candidates1.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates1 {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates_a {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1.0, code_a.0)) {
                    cost_old = cost_old.saturating_add(mod_val);
                    cost_new = cost_new.saturating_add(mod_val);
                }

                if cost_old < min_old { min_old = cost_old; }
                if cost_new < min_new { min_new = cost_new; }
            }
        }
        delta += (min_new.0 - min_old.0) * i64::from(ctx.bigram_rev_freqs[k]);
    }

    // Bigrams(x, b) where x != a, x != b
    let start_rev_b = ctx.bigram_rev_starts[code_b.0 as usize];
    let end_rev_b = ctx.bigram_rev_starts[code_b.0 as usize + 1];
    for k in start_rev_b..end_rev_b {
        let c1 = ctx.bigram_rev_others[k];
        if c1 == code_a || c1 == code_b { continue; }
        let candidates1 = pos_map.get(c1.0 as usize);
        if candidates1.is_empty() { continue; }

        let mut min_old = Score(i64::MAX);
        let mut min_new = Score(i64::MAX);
        for &p1 in candidates1 {
            let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
            for &p2 in candidates_b {
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                
                let mut cost_old = ctx.cost_matrix[(p1 as usize) * ctx.key_count + (p2 as usize)];
                let mut cost_new = ctx.cost_matrix[p1_new * ctx.key_count + p2_new];

                if let Some(&mod_val) = ctx.sequence_modifiers.get(&(c1.0, code_b.0)) {
                    cost_old = cost_old.saturating_add(mod_val);
                    cost_new = cost_new.saturating_add(mod_val);
                }

                if cost_old < min_old { min_old = cost_old; }
                if cost_new < min_new { min_new = cost_new; }
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
            if c1 == code_a || c1 == code_b { continue; }
            let c3 = ctx.trigram_mid_others2[k];
            let freq = i64::from(ctx.trigram_mid_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, code_a, c3, idx_a, idx_b) * freq;
        }

        // Mid(b) where c1 != a and c1 != b
        let s_mb = ctx.trigram_mid_starts[cb];
        let e_mb = ctx.trigram_mid_starts[cb + 1];
        for k in s_mb..e_mb {
            let c1 = ctx.trigram_mid_others1[k];
            if c1 == code_a || c1 == code_b { continue; }
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
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b { continue; }
            let freq = i64::from(ctx.trigram_end_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_a, idx_a, idx_b) * freq;
        }

        // Ends(b) where c1 != a, c1 != b, c2 != a, c2 != b
        let s_eb = ctx.trigram_end_starts[cb];
        let e_eb = ctx.trigram_end_starts[cb + 1];
        for k in s_eb..e_eb {
            let c1 = ctx.trigram_end_others1[k];
            let c2 = ctx.trigram_end_others2[k];
            if c1 == code_a || c1 == code_b || c2 == code_a || c2 == code_b { continue; }
            let freq = i64::from(ctx.trigram_end_freqs[k]);
            delta += get_flow_delta(ctx, pos_map, c1, c2, code_b, idx_a, idx_b) * freq;
        }
    }

    delta
}
