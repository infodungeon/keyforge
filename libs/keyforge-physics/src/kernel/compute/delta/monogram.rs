// libs/keyforge-physics/src/kernel/compute/delta/monogram.rs

use super::get_p_effective;
use super::PosMap;
use crate::kernel::{types::Score, EngineContext};

#[allow(clippy::cast_possible_wrap)]
pub(crate) fn calculate_monogram_delta(
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

    let candidates_a = pos_map.get(code_a);
    let candidates_b = pos_map.get(code_b);

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
