// libs/keyforge-physics/src/kernel/compute/delta/bigram.rs

use super::get_p_effective;
use super::PosMap;
use crate::kernel::{types::Score, EngineContext};

#[allow(clippy::similar_names)]
pub(crate) fn calculate_bigram_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    code_a: crate::kernel::types::KeyCode,
    code_b: crate::kernel::types::KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let mut delta = 0i64;
    let ca_val = code_a.0 as usize;
    let cb_val = code_b.0 as usize;
    let candidates_a = pos_map.get(code_a);
    let candidates_b = pos_map.get(code_b);

    // Bigrams(a, x)
    if ca_val + 1 < ctx.corpus.bigram_starts.len() {
        let start = ctx.corpus.bigram_starts[ca_val];
        let end = ctx.corpus.bigram_starts[ca_val + 1];
        for k in start..end {
            let c2 = ctx.corpus.bigram_others[k];
            delta += get_pair_delta(ctx, code_a, c2, candidates_a, pos_map.get(c2), idx_a, idx_b)
                * i64::from(ctx.corpus.bigram_freqs[k]);
        }
    }

    // Bigrams(b, x)
    if cb_val + 1 < ctx.corpus.bigram_starts.len() {
        let start = ctx.corpus.bigram_starts[cb_val];
        let end = ctx.corpus.bigram_starts[cb_val + 1];
        for k in start..end {
            let c2 = ctx.corpus.bigram_others[k];
            delta += get_pair_delta(ctx, code_b, c2, candidates_b, pos_map.get(c2), idx_a, idx_b)
                * i64::from(ctx.corpus.bigram_freqs[k]);
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
            delta += get_pair_delta(ctx, c1, code_a, pos_map.get(c1), candidates_a, idx_a, idx_b)
                * i64::from(ctx.corpus.bigram_rev_freqs[k]);
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
            delta += get_pair_delta(ctx, c1, code_b, pos_map.get(c1), candidates_b, idx_a, idx_b)
                * i64::from(ctx.corpus.bigram_rev_freqs[k]);
        }
    }

    delta
}

#[allow(clippy::too_many_arguments)]
fn get_pair_delta(
    ctx: &EngineContext,
    c1: crate::kernel::types::KeyCode,
    c2: crate::kernel::types::KeyCode,
    cand1: &[crate::kernel::types::KeyIndex],
    cand2: &[crate::kernel::types::KeyIndex],
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    if cand1.is_empty() || cand2.is_empty() {
        return 0;
    }
    let mut min_old = Score::INFINITY_SENTINEL;
    let mut min_new = Score::INFINITY_SENTINEL;
    for &p1 in cand1 {
        let p1_new = get_p_effective(p1.as_usize(), idx_a, idx_b);
        for &p2 in cand2 {
            let p2_new = get_p_effective(p2.as_usize(), idx_a, idx_b);
            let mut cost_old =
                ctx.geometry.cost_matrix[p1.as_usize() * ctx.key_count + p2.as_usize()];
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
