use crate::kernel::{EngineContext, types::{Score, KeyCode}};
use super::state::PosMap;

#[inline(always)]
pub(crate) fn calculate_flow_cost(ctx: &EngineContext, p1: usize, p2: usize, p3: usize) -> Score {
    let h1 = ctx.hands[p1];
    let h2 = ctx.hands[p2];
    let h3 = ctx.hands[p3];
    if h1 != h2 || h2 != h3 { return Score::ZERO; }

    if ctx.fingers[p1] == ctx.fingers[p3] && ctx.fingers[p1] != ctx.fingers[p2] { return ctx.penalty_redirect; }
    
    let dir1 = ctx.fingers[p2].diff(ctx.fingers[p1]);
    let dir2 = ctx.fingers[p3].diff(ctx.fingers[p2]);
    if dir1 == 0 || dir2 == 0 { return Score::ZERO; }
    if dir1.signum() != dir2.signum() { return ctx.penalty_redirect; }
    if dir1 < 0 { return Score::ZERO.saturating_sub(ctx.bonus_roll); }
    Score::ZERO
}

#[inline(always)]
pub(crate) fn get_p_effective(p: usize, idx_a: usize, idx_b: usize) -> usize {
    if p == idx_a {
        idx_b
    } else if p == idx_b {
        idx_a
    } else {
        p
    }
}

#[inline(always)]
pub(crate) fn get_flow_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    c1: KeyCode,
    c2: KeyCode,
    c3: KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let candidates1 = pos_map.get(c1.0 as usize);
    let candidates2 = pos_map.get(c2.0 as usize);
    let candidates3 = pos_map.get(c3.0 as usize);
    if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() {
        return 0;
    }

    let mut min_old = Score(i64::MAX);
    for &p1 in candidates1 {
        for &p2 in candidates2 {
            for &p3 in candidates3 {
                let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                if cost < min_old { min_old = cost; }
            }
        }
    }

    let mut min_new = Score(i64::MAX);
    for &p1 in candidates1 {
        for &p2 in candidates2 {
            for &p3 in candidates3 {
                let p1_new = get_p_effective(p1 as usize, idx_a, idx_b);
                let p2_new = get_p_effective(p2 as usize, idx_a, idx_b);
                let p3_new = get_p_effective(p3 as usize, idx_a, idx_b);
                let cost = calculate_flow_cost(ctx, p1_new, p2_new, p3_new);
                if cost < min_new { min_new = cost; }
            }
        }
    }

    min_new.0 - min_old.0
}
