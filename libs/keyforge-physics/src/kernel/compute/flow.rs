use super::state::PosMap;
use crate::kernel::mechanics::calculate_flow_cost as shared_calculate_flow_cost;
use crate::kernel::{
    types::{KeyCode, Score},
    EngineContext,
};

#[inline]
pub(crate) fn calculate_flow_cost(ctx: &EngineContext, p1: usize, p2: usize, p3: usize) -> Score {
    shared_calculate_flow_cost(
        ctx.geometry.hands[p1],
        ctx.geometry.hands[p2],
        ctx.geometry.hands[p3],
        ctx.geometry.fingers[p1],
        ctx.geometry.fingers[p2],
        ctx.geometry.fingers[p3],
        ctx.penalty_redirect,
        ctx.bonus_roll,
        ctx.bonus_roll_out,
    )
}

#[inline]
pub(crate) fn get_p_effective(p: usize, idx_a: usize, idx_b: usize) -> usize {
    if p == idx_a {
        idx_b
    } else if p == idx_b {
        idx_a
    } else {
        p
    }
}

#[inline]
pub(crate) fn get_flow_delta(
    ctx: &EngineContext,
    pos_map: &PosMap<'_>,
    c1: KeyCode,
    c2: KeyCode,
    c3: KeyCode,
    idx_a: usize,
    idx_b: usize,
) -> i64 {
    let candidates1 = pos_map.get(c1);
    let candidates2 = pos_map.get(c2);
    let candidates3 = pos_map.get(c3);
    if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() {
        return 0;
    }

    let mut min_old = Score(i64::MAX);
    for &p1 in candidates1 {
        for &p2 in candidates2 {
            for &p3 in candidates3 {
                let cost = calculate_flow_cost(ctx, p1 as usize, p2 as usize, p3 as usize);
                if cost < min_old {
                    min_old = cost;
                }
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
                if cost < min_new {
                    min_new = cost;
                }
            }
        }
    }

    min_new.0 - min_old.0
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::kernel::EngineContext;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};

    #[cfg(test)]
    fn setup_mock_flow_ctx() -> EngineContext {
        use std::sync::Arc;
        EngineContext {
            key_count: 3,
            geometry: crate::kernel::GeometryData {
                hands: Arc::new([HandIndex::LEFT; 3]),
                fingers: Arc::new([FingerIndex::INDEX, FingerIndex::MIDDLE, FingerIndex::RING]),
                rows: Arc::new([RowIndex(0); 3]),
                cols: Arc::new([ColIndex(0), ColIndex(1), ColIndex(2)]),
                cost_matrix: Arc::new([]),
                dist_matrix: Arc::new([]),
                key_home_distances: Arc::new([]),
                key_costs: Arc::new([]),
            },
            corpus: crate::kernel::CorpusData {
                char_freqs: Arc::new([]),
                bigram_starts: Arc::new([]),
                bigram_others: Arc::new([]),
                bigram_freqs: Arc::new([]),
                bigram_rev_starts: Arc::new([]),
                bigram_rev_others: Arc::new([]),
                bigram_rev_freqs: Arc::new([]),
                trigram_starts: Arc::new([]),
                trigram_others1: Arc::new([]),
                trigram_others2: Arc::new([]),
                trigram_freqs: Arc::new([]),
                trigram_mid_starts: Arc::new([]),
                trigram_mid_others1: Arc::new([]),
                trigram_mid_others2: Arc::new([]),
                trigram_mid_freqs: Arc::new([]),
                trigram_end_starts: Arc::new([]),
                trigram_end_others1: Arc::new([]),
                trigram_end_others2: Arc::new([]),
                trigram_end_freqs: Arc::new([]),
            },
            all_bigrams: Arc::new([]),
            all_trigrams: Arc::new([]),
            penalty_redirect: Score(100),
            bonus_roll: Score(50),
            bonus_roll_out: Score(50),
            sequence_modifiers: Arc::new(std::collections::HashMap::new()),
        }
    }

    #[test]
    fn test_calculate_flow_cost_roll() {
        let ctx = setup_mock_flow_ctx();
        // Index -> Middle -> Ring (Outward in our coordinate system? diff > 0)
        // Wait, calculate_flow_cost says: if dir1 < 0 { return sub(bonus_roll) }
        // diff(Middle, Index) = 2 - 1 = 1.
        // So 1 -> 2 -> 3 is outward (positive diff).
        // 3 -> 2 -> 1 is inward (negative diff).

        let cost = calculate_flow_cost(&ctx, 2, 1, 0); // Ring -> Middle -> Index
        assert_eq!(cost.0, -50);
    }

    #[test]
    fn test_calculate_flow_cost_redirect() {
        let ctx = setup_mock_flow_ctx();
        let cost = calculate_flow_cost(&ctx, 0, 1, 0); // Index -> Middle -> Index
        assert_eq!(cost.0, 100);
    }
}
