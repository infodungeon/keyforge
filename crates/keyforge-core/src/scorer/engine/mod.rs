mod bigrams;
mod monograms;
mod trigrams;

use super::{ScoreDetails, Scorer};
use crate::consts::KEY_CODE_RANGE;

pub use bigrams::score_bigrams;
pub use monograms::score_monograms;
pub use trigrams::score_trigrams;

pub fn score_full(
    scorer: &Scorer,
    pos_map: &[u8; KEY_CODE_RANGE],
    limit: usize,
) -> (f32, f32, f32) {
    let mut score = 0.0;
    let mut left_load = 0.0;
    let mut total_freq = 0.0;

    monograms::score_monograms(scorer, pos_map, &mut score, &mut left_load, &mut total_freq);
    bigrams::score_bigrams(scorer, pos_map, &mut score);
    trigrams::score_trigrams(scorer, pos_map, &mut score, limit);

    (score, left_load, total_freq)
}

pub fn score_details(
    scorer: &Scorer,
    pos_map: &[u8; KEY_CODE_RANGE],
    limit: usize,
) -> ScoreDetails {
    let mut d = ScoreDetails::default();

    // 1. Accumulate Stats from modules
    monograms::accumulate_details(scorer, pos_map, &mut d);
    bigrams::accumulate_details(scorer, pos_map, &mut d);
    trigrams::accumulate_details(scorer, pos_map, &mut d, limit);

    // 2. Final Heuristics
    let mut left_load = 0.0;
    let mut total_freq = 0.0;

    for &i in &scorer.active_chars {
        let p = pos_map[i];
        if p != crate::consts::KEY_NOT_FOUND_U8 {
            let freq = scorer.char_freqs[i];
            total_freq += freq;
            let info = &scorer.geometry.keys[p as usize];
            if info.hand == 0 {
                left_load += freq;
            }
        }
    }

    if total_freq > 0.0 {
        let ratio = left_load / total_freq;
        let diff = (ratio - 0.5).abs();
        let allowed = scorer.weights.allowed_hand_balance_deviation();
        if diff > allowed {
            d.imbalance_penalty = diff * scorer.weights.penalty_imbalance;
        }
    }

    d.layout_score = d.geo_dist
        + d.finger_use
        + d.mech_sfb
        + d.mech_sfb_lat
        + d.mech_sfb_lat_weak
        + d.mech_sfb_diag
        + d.mech_sfb_long
        + d.mech_sfb_bot
        + d.mech_scis
        + d.mech_lat
        + d.mech_sfr
        + d.flow_cost
        + d.tier_penalty
        + d.imbalance_penalty
        + d.mech_mono_stretch;

    d
}

pub fn calculate_key_costs(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE]) -> Vec<f32> {
    let mut costs = vec![0.0; scorer.key_count];

    monograms::accumulate_key_costs(scorer, pos_map, &mut costs);
    bigrams::accumulate_key_costs(scorer, pos_map, &mut costs);
    trigrams::accumulate_key_costs(scorer, pos_map, &mut costs);

    costs
}
