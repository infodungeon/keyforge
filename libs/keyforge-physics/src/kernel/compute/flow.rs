use crate::kernel::mechanics::calculate_flow_cost as shared_calculate_flow_cost;
use crate::kernel::{types::Score, EngineContext};

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
                rows: Arc::new([RowIndex::new(0); 3]),
                cols: Arc::new([ColIndex::new(0), ColIndex::new(1), ColIndex::new(2)]),
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
            },
            all_bigrams: Arc::new([]),
            all_trigrams: Arc::new([]),
            penalty_redirect: Score::from_scaled_i64(100),
            bonus_roll: Score::from_scaled_i64(50),
            bonus_roll_out: Score::from_scaled_i64(50),
            sequence_modifiers: Arc::new(std::collections::HashMap::new()),
        }
    }

    #[test]
    fn test_calculate_flow_cost_roll() {
        let ctx = setup_mock_flow_ctx();
        let cost = calculate_flow_cost(&ctx, 2, 1, 0); // Ring -> Middle -> Index
        assert_eq!(cost.raw(), -50);
    }

    #[test]
    fn test_calculate_flow_cost_redirect() {
        let ctx = setup_mock_flow_ctx();
        let cost = calculate_flow_cost(&ctx, 0, 1, 0); // Index -> Middle -> Index
        assert_eq!(cost.raw(), 100);
    }
}
