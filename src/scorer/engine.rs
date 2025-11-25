use super::flow::analyze_flow;
use super::physics::{analyze_interaction, get_geo_dist, get_reach_cost};
use super::{ScoreDetails, Scorer};

pub fn score_full(scorer: &Scorer, pos_map: &[u8; 256], limit: usize) -> (f32, f32, f32) {
    let mut score = 0.0;
    let mut left_load = 0.0;
    let mut total_freq = 0.0;

    for i in 0..256 {
        let freq = scorer.char_freqs[i];
        if freq > 0.0 {
            let p = pos_map[i];
            if p != 255 {
                total_freq += freq;
                if scorer.geometry.keys[p as usize].hand == 0 {
                    left_load += freq;
                }
                score += scorer.tier_penalty_matrix[scorer.char_tier_map[i] as usize]
                    [scorer.slot_tier_map[p as usize] as usize]
                    * freq;
                score += scorer.slot_monogram_costs[p as usize] * freq;
            }
        }
    }

    for c1 in 0..256 {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 != 255 {
                    score +=
                        scorer.full_cost_matrix[p1 as usize][p2 as usize] * scorer.bigrams_freqs[k];
                }
            }
        }
    }

    for c1 in 0..256 {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let start = scorer.trigram_starts[c1];
        let end = scorer.trigram_starts[c1 + 1];
        let effective_end = if limit > 0 && (end - start) > limit {
            start + limit
        } else {
            end
        };
        for k in start..effective_end {
            let t = &scorer.trigrams_flat[k];
            if t.role == 0 {
                let c2 = t.other1 as usize;
                let c3 = t.other2 as usize;
                let p2 = pos_map[c2];
                let p3 = pos_map[c3];
                if p2 != 255 && p3 != 255 {
                    let idx = (p1 as usize) * 900 + (p2 as usize) * 30 + (p3 as usize);
                    let cost = scorer.trigram_cost_table[idx];
                    if cost != 0.0 {
                        score += cost * t.freq;
                    }
                }
            }
        }
    }
    (score, left_load, total_freq)
}

pub fn score_debug(scorer: &Scorer, pos_map: &[u8; 256], limit: usize) -> ScoreDetails {
    let mut d = ScoreDetails::default();
    let mut total_freq = 0.0;
    let mut left_load = 0.0;

    for i in 0..256 {
        let freq = scorer.char_freqs[i];
        if freq > 0.0 {
            let p = pos_map[i];
            if p != 255 {
                total_freq += freq;
                let info = &scorer.geometry.keys[p as usize];
                if info.hand == 0 {
                    left_load += freq;
                }
                d.tier_penalty += scorer.tier_penalty_matrix[scorer.char_tier_map[i] as usize]
                    [scorer.slot_tier_map[p as usize] as usize]
                    * freq;
                d.finger_use += (scorer.finger_scales[info.finger as usize]
                    * scorer.weights.weight_finger_effort)
                    * freq;
                d.geo_dist +=
                    get_reach_cost(&scorer.geometry, p as usize, scorer.weights.weight_geo_dist)
                        * freq;
            }
        }
    }

    for c1 in 0..256 {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 != 255 {
                    let freq = scorer.bigrams_freqs[k];
                    d.user_dist += scorer.raw_user_matrix[p1 as usize][p2 as usize] * freq;

                    let m = analyze_interaction(&scorer.geometry, p1 as usize, p2 as usize);

                    if m.is_same_hand {
                        let dist = get_geo_dist(
                            &scorer.geometry,
                            p1 as usize,
                            p2 as usize,
                            scorer.weights.weight_geo_dist,
                        );
                        d.geo_dist += dist * freq;

                        let mut penalty = 1.0;
                        let mut weak_applied = false;

                        if m.is_repeat {
                            d.mech_sfr += (dist * 5.0) * freq;
                        } else if m.is_sfb {
                            // Reporting Buckets
                            if m.is_lat_step {
                                if m.is_strong_finger {
                                    d.mech_sfb_lat +=
                                        (dist * scorer.weights.penalty_sfb_lateral) * freq;
                                    penalty = scorer.weights.penalty_sfb_lateral;
                                } else {
                                    d.mech_sfb_lat_weak +=
                                        (dist * scorer.weights.penalty_sfb_lateral_weak) * freq;
                                    penalty = scorer.weights.penalty_sfb_lateral_weak;
                                    weak_applied = true;
                                }
                            } else if m.is_bot_lat_seq {
                                d.mech_sfb_bot += (dist * scorer.weights.penalty_sfb_bottom) * freq;
                                penalty = scorer.weights.penalty_sfb_bottom;
                            } else if m.row_diff >= 2 {
                                d.mech_sfb_long += (dist * scorer.weights.penalty_sfb_long) * freq;
                                penalty = scorer.weights.penalty_sfb_long;
                            } else if m.row_diff > 0 && m.col_diff > 0 {
                                d.mech_sfb_diag +=
                                    (dist * scorer.weights.penalty_sfb_diagonal) * freq;
                                penalty = scorer.weights.penalty_sfb_diagonal;
                            } else {
                                d.mech_sfb += (dist * scorer.weights.penalty_sfb_base) * freq;
                                penalty = scorer.weights.penalty_sfb_base;
                                if m.is_outward {
                                    penalty += scorer.weights.penalty_sfb_outward_adder;
                                }
                            }

                            if !m.is_strong_finger && !weak_applied {
                                penalty *= scorer.weights.weight_weak_finger_sfb;
                            }
                        } else if m.is_scissor {
                            d.mech_scis += (dist * scorer.weights.penalty_scissor) * freq;
                            penalty = scorer.weights.penalty_scissor;
                        } else if m.is_lateral_stretch {
                            d.mech_lat += (dist * scorer.weights.penalty_lateral) * freq;
                            penalty = scorer.weights.penalty_lateral;
                        }

                        if penalty > 1.0 {
                            d.user_dist += (scorer.raw_user_matrix[p1 as usize][p2 as usize]
                                * (penalty - 1.0))
                                * freq;
                        }
                    }
                }
            }
        }
    }

    for c1 in 0..256 {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let start = scorer.trigram_starts[c1];
        let end = scorer.trigram_starts[c1 + 1];
        let effective_end = if limit > 0 && (end - start) > limit {
            start + limit
        } else {
            end
        };
        for k in start..effective_end {
            let t = &scorer.trigrams_flat[k];
            if t.role == 0 {
                let c2 = t.other1 as usize;
                let c3 = t.other2 as usize;
                let p2 = pos_map[c2];
                let p3 = pos_map[c3];
                if p2 != 255 && p3 != 255 {
                    let idx = (p1 as usize) * 900 + (p2 as usize) * 30 + (p3 as usize);
                    let cost = scorer.trigram_cost_table[idx];
                    if cost != 0.0 {
                        d.flow_cost += cost * t.freq;
                        let k1 = &scorer.geometry.keys[p1 as usize];
                        let k2 = &scorer.geometry.keys[p2 as usize];
                        let k3 = &scorer.geometry.keys[p3 as usize];
                        let flow = analyze_flow(k1, k2, k3);
                        if flow.is_3_hand_run {
                            if flow.is_skip {
                                d.flow_skip += scorer.weights.penalty_skip * t.freq;
                            } else if flow.is_redirect {
                                d.flow_redirect += scorer.weights.penalty_redirect * t.freq;
                            } else if flow.is_inward_roll {
                                d.flow_roll += scorer.weights.bonus_inward_roll * t.freq;
                            }
                        }
                    }
                }
            }
        }
    }

    if total_freq > 0.0 {
        let ratio = left_load / total_freq;
        let dist = (ratio - 0.5).abs();
        if dist > (scorer.weights.max_hand_imbalance - 0.5) {
            d.imbalance_penalty = dist * scorer.weights.penalty_imbalance;
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
        + d.imbalance_penalty;

    d
}
