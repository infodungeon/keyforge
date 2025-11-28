use super::flow::analyze_flow;
use super::physics::{analyze_interaction, get_geo_dist, get_reach_cost};
use super::{ScoreDetails, Scorer};

/// Fast Path: Used by the Optimizer.
pub fn score_full(scorer: &Scorer, pos_map: &[u8; 256], limit: usize) -> (f32, f32, f32) {
    let mut score = 0.0;
    let mut left_load = 0.0;
    let mut total_freq = 0.0;

    // 1. Chars
    for (i, &p) in pos_map.iter().enumerate() {
        let freq = scorer.char_freqs[i];
        if freq > 0.0 && p != 255 {
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

    // 2. Bigrams
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

    // 3. Trigrams
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

/// Detailed Path: Used by Validation.
pub fn score_debug(scorer: &Scorer, pos_map: &[u8; 256], limit: usize) -> ScoreDetails {
    let mut d = ScoreDetails::default();
    let mut total_freq = 0.0;
    let mut left_load = 0.0;

    // 1. CHARS
    for (i, &p) in pos_map.iter().enumerate() {
        let freq = scorer.char_freqs[i];
        if freq > 0.0 {
            d.total_chars += freq;
            if p != 255 {
                total_freq += freq;
                let info = &scorer.geometry.keys[p as usize];
                if info.hand == 0 {
                    left_load += freq;
                }

                // Stat: Pinky Reach (Finger 4, Not Home Row 1)
                if info.finger == 4 && info.row != 1 {
                    d.stat_pinky_reach += freq;
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

    // 2. BIGRAMS
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
                    d.total_bigrams += freq;
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

                        // === STATS COLLECTION ===
                        if m.is_sfb {
                            d.stat_sfb += freq;
                        }
                        if m.is_scissor {
                            d.stat_scis += freq;
                        }

                        // LSB Logic: Combine SFB Lat and Non-SFB Lat for stats
                        if m.is_lat_step || m.is_lateral_stretch {
                            d.stat_lsb += freq;
                        }
                        // Track Non-SFB Lat specifically as well
                        if m.is_lateral_stretch {
                            d.stat_lat += freq;
                        }

                        // Rolls (Bigram Level) Stats
                        if m.is_roll_in {
                            d.stat_roll_in += freq;
                            d.stat_roll += freq;
                        } else if m.is_roll_out {
                            d.stat_roll_out += freq;
                            d.stat_roll += freq;
                        }

                        // === SCORING LOGIC (Weighted) ===

                        // Flow Bonuses (Bigram)
                        if m.is_roll_in {
                            let bonus = scorer.weights.bonus_bigram_roll_in * freq;
                            d.flow_roll_in += bonus;
                            d.flow_cost -= bonus;
                        } else if m.is_roll_out {
                            let bonus = scorer.weights.bonus_bigram_roll_out * freq;
                            d.flow_roll_out += bonus;
                            d.flow_cost -= bonus;
                        }

                        // Penalties
                        let mut penalty = 1.0;
                        let mut weak_applied = false;

                        if m.is_repeat {
                            d.stat_sfr += freq;
                            let mut sfr_cost = 0.0;
                            if m.is_strong_finger {
                                if !m.is_home_row {
                                    sfr_cost += scorer.weights.penalty_sfr_bad_row;
                                    if m.is_stretch_col {
                                        sfr_cost += scorer.weights.penalty_sfr_lat
                                            - scorer.weights.penalty_sfr_bad_row;
                                    }
                                }
                            } else if m.is_home_row {
                                sfr_cost += scorer.weights.penalty_sfr_weak_finger;
                            } else {
                                sfr_cost += scorer.weights.penalty_sfr_bad_row * 5.0;
                            }
                            d.mech_sfr += sfr_cost * freq;
                        } else if m.is_sfb {
                            // SFB Breakdown
                            if m.is_bot_lat_seq {
                                d.stat_sfb_bot += freq;
                                d.mech_sfb_bot += (dist * scorer.weights.penalty_sfb_bottom) * freq;
                                penalty = scorer.weights.penalty_sfb_bottom;
                            } else if m.row_diff >= 2 {
                                d.stat_sfb_long += freq;
                                d.mech_sfb_long += (dist * scorer.weights.penalty_sfb_long) * freq;
                                penalty = scorer.weights.penalty_sfb_long;
                            } else if m.row_diff > 0 && m.col_diff > 0 {
                                d.stat_sfb_diag += freq;
                                d.mech_sfb_diag +=
                                    (dist * scorer.weights.penalty_sfb_diagonal) * freq;
                                penalty = scorer.weights.penalty_sfb_diagonal;
                            } else if m.is_lat_step {
                                if m.is_strong_finger {
                                    d.stat_sfb_lat += freq;
                                    d.mech_sfb_lat +=
                                        (dist * scorer.weights.penalty_sfb_lateral) * freq;
                                    penalty = scorer.weights.penalty_sfb_lateral;
                                } else {
                                    d.stat_sfb_lat_weak += freq;
                                    d.mech_sfb_lat_weak +=
                                        (dist * scorer.weights.penalty_sfb_lateral_weak) * freq;
                                    penalty = scorer.weights.penalty_sfb_lateral_weak;
                                    weak_applied = true;
                                }
                            } else {
                                d.stat_sfb_base += freq;
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

    // 3. TRIGRAMS
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
                            if flow.is_redirect {
                                d.stat_redir += t.freq;
                                d.flow_redirect += scorer.weights.penalty_redirect * t.freq;
                            } else if flow.is_skip {
                                d.stat_skip += t.freq;
                                d.flow_skip += scorer.weights.penalty_skip * t.freq;
                            } else if flow.is_inward_roll {
                                d.stat_roll3_in += t.freq;
                                let bonus = scorer.weights.bonus_inward_roll * t.freq;
                                d.flow_roll_tri += bonus;
                                d.flow_cost -= bonus;
                            } else if flow.is_outward_roll {
                                d.stat_roll3_out += t.freq;
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
