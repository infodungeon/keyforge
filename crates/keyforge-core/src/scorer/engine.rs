// ===== keyforge/src/scorer/engine.rs =====
use super::costs::{calculate_cost, CostCategory};
use super::flow::analyze_flow;
use super::physics::{analyze_interaction, get_geo_dist, get_reach_cost};
use super::{ScoreDetails, Scorer};

pub fn score_full(scorer: &Scorer, pos_map: &[u8; 256], limit: usize) -> (f32, f32, f32) {
    // ... (content remains same)
    let mut score = 0.0;
    let mut left_load = 0.0;
    let mut total_freq = 0.0;

    // OPTIMIZATION: Iterate only active chars
    for &c_idx in &scorer.active_chars {
        let p = pos_map[c_idx];
        if p != 255 {
            let p_idx = p as usize;
            if p_idx >= scorer.key_count {
                continue;
            }

            let freq = scorer.char_freqs[c_idx];

            total_freq += freq;
            if scorer.geometry.keys[p_idx].hand == 0 {
                left_load += freq;
            }
            score += scorer.tier_penalty_matrix[scorer.char_tier_map[c_idx] as usize]
                [scorer.slot_tier_map[p_idx] as usize]
                * freq;
            score += scorer.slot_monogram_costs[p_idx] * freq;
        }
    }

    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 != 255 {
                    let p2_idx = p2 as usize;
                    let idx = p1_idx * scorer.key_count + p2_idx;
                    score += scorer.full_cost_matrix[idx] * scorer.bigrams_freqs[k];
                }
            }
        }
    }

    let k_sq = scorer.key_count * scorer.key_count;

    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let p1_idx = p1 as usize;

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
                    let p2_idx = p2 as usize;
                    let p3_idx = p3 as usize;
                    let idx = p1_idx * k_sq + p2_idx * scorer.key_count + p3_idx;
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

// RENAMED HERE
pub fn score_details(scorer: &Scorer, pos_map: &[u8; 256], limit: usize) -> ScoreDetails {
    let mut d = ScoreDetails::default();
    let mut total_freq = 0.0;
    let mut left_load = 0.0;

    // 1. CHARS
    for &i in &scorer.active_chars {
        let p = pos_map[i];
        let freq = scorer.char_freqs[i];
        if freq > 0.0 {
            d.total_chars += freq;
            if p != 255 {
                let p_idx = p as usize;
                if p_idx >= scorer.key_count {
                    continue;
                }

                total_freq += freq;
                let info = &scorer.geometry.keys[p_idx];
                if info.hand == 0 {
                    left_load += freq;
                }

                if info.finger == 4 && info.row != scorer.geometry.home_row {
                    d.stat_pinky_reach += freq;
                }

                d.tier_penalty += scorer.tier_penalty_matrix[scorer.char_tier_map[i] as usize]
                    [scorer.slot_tier_map[p_idx] as usize]
                    * freq;

                if info.is_stretch {
                    d.stat_mono_stretch += freq;
                    d.mech_mono_stretch += scorer.weights.penalty_monogram_stretch * freq;
                }

                d.finger_use += (scorer.finger_scales[info.finger as usize]
                    * scorer.weights.weight_finger_effort)
                    * freq;

                d.geo_dist += get_reach_cost(
                    &scorer.geometry,
                    p_idx,
                    scorer.weights.weight_lateral_travel,
                    scorer.weights.weight_vertical_travel,
                ) * freq;
            }
        }
    }

    // 2. BIGRAMS
    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 != 255 {
                    let p2_idx = p2 as usize;
                    let freq = scorer.bigrams_freqs[k];
                    d.total_bigrams += freq;

                    let flat_idx = p1_idx * scorer.key_count + p2_idx;
                    d.user_dist += scorer.raw_user_matrix[flat_idx] * freq;

                    let m = analyze_interaction(&scorer.geometry, p1_idx, p2_idx, &scorer.weights);

                    if m.is_same_hand {
                        let dist = get_geo_dist(
                            &scorer.geometry,
                            p1_idx,
                            p2_idx,
                            scorer.weights.weight_lateral_travel,
                            scorer.weights.weight_vertical_travel,
                        );
                        d.geo_dist += dist * freq;

                        if m.is_sfb {
                            d.stat_sfb += freq;
                        }
                        if m.is_scissor {
                            d.stat_scis += freq;
                        }
                        if m.is_lat_step || m.is_lateral_stretch {
                            d.stat_lsb += freq;
                        }
                        if m.is_lateral_stretch {
                            d.stat_lat += freq;
                        }
                        if m.is_roll_in {
                            d.stat_roll_in += freq;
                            d.stat_roll += freq;
                        } else if m.is_roll_out {
                            d.stat_roll_out += freq;
                            d.stat_roll += freq;
                        }

                        let res = calculate_cost(&m, &scorer.weights);

                        match res.category {
                            CostCategory::SfbBase => d.stat_sfb_base += freq,
                            CostCategory::SfbLat => d.stat_sfb_lat += freq,
                            CostCategory::SfbLatWeak => d.stat_sfb_lat_weak += freq,
                            CostCategory::SfbDiag => d.stat_sfb_diag += freq,
                            CostCategory::SfbLong => d.stat_sfb_long += freq,
                            CostCategory::SfbBot => d.stat_sfb_bot += freq,
                            CostCategory::SfrBase
                            | CostCategory::SfrBadRow
                            | CostCategory::SfrLat
                            | CostCategory::SfrWeak => d.stat_sfr += freq,
                            _ => {}
                        }

                        if res.flow_bonus > 0.0 {
                            if m.is_roll_in {
                                d.flow_roll_in += res.flow_bonus * freq;
                            } else if m.is_roll_out {
                                d.flow_roll_out += res.flow_bonus * freq;
                            }
                            d.flow_cost -= res.flow_bonus * freq;
                        }

                        if res.additive_cost > 0.0 {
                            d.mech_sfr += res.additive_cost * freq;
                        }

                        if res.penalty_multiplier > 1.0 {
                            d.user_dist += (scorer.raw_user_matrix[flat_idx]
                                * (res.penalty_multiplier - 1.0))
                                * freq;
                            let cost_val = (dist * res.penalty_multiplier) * freq;
                            match res.category {
                                CostCategory::SfbBase => d.mech_sfb += cost_val,
                                CostCategory::SfbLat | CostCategory::SfbLatWeak => {
                                    if m.is_strong_finger {
                                        d.mech_sfb_lat += cost_val;
                                    } else {
                                        d.mech_sfb_lat_weak += cost_val;
                                    }
                                }
                                CostCategory::SfbDiag => d.mech_sfb_diag += cost_val,
                                CostCategory::SfbLong => d.mech_sfb_long += cost_val,
                                CostCategory::SfbBot => d.mech_sfb_bot += cost_val,
                                CostCategory::Scissor => d.mech_scis += cost_val,
                                CostCategory::Lateral => d.mech_lat += cost_val,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    let k_sq = scorer.key_count * scorer.key_count;

    // 3. TRIGRAMS
    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.trigram_starts[c1];
        let end = scorer.trigram_starts[c1 + 1];
        let effective_end = if limit > 0 && (end - start) > limit {
            start + limit
        } else {
            end
        };

        for k in start..effective_end {
            let t = &scorer.trigrams_flat[k];
            d.total_trigrams += t.freq;

            if t.role == 0 {
                let c2 = t.other1 as usize;
                let c3 = t.other2 as usize;
                let p2 = pos_map[c2];
                let p3 = pos_map[c3];
                if p2 != 255 && p3 != 255 {
                    let p2_idx = p2 as usize;
                    let p3_idx = p3 as usize;

                    let idx = p1_idx * k_sq + p2_idx * scorer.key_count + p3_idx;
                    let cost = scorer.trigram_cost_table[idx];

                    if cost != 0.0 {
                        d.flow_cost += cost * t.freq;
                        let k1 = &scorer.geometry.keys[p1_idx];
                        let k2 = &scorer.geometry.keys[p2_idx];
                        let k3 = &scorer.geometry.keys[p3_idx];
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

// ... (calculate_key_costs remains same) ...
pub fn calculate_key_costs(scorer: &Scorer, pos_map: &[u8; 256]) -> Vec<f32> {
    // ... same as before
    let mut costs = vec![0.0; scorer.key_count];

    // 1. Monogram Costs (Effort + Travel)
    for &c_idx in &scorer.active_chars {
        let p = pos_map[c_idx];
        if p != 255 {
            let p_idx = p as usize;
            if p_idx >= scorer.key_count {
                continue;
            }
            let freq = scorer.char_freqs[c_idx];

            // Add base effort/travel cost
            costs[p_idx] += scorer.slot_monogram_costs[p_idx] * freq;

            // Add Tier penalty
            let char_tier = scorer.char_tier_map[c_idx] as usize;
            let slot_tier = scorer.slot_tier_map[p_idx] as usize;
            costs[p_idx] += scorer.tier_penalty_matrix[char_tier][slot_tier] * freq;
        }
    }

    // 2. Bigram Costs (SFBs, flow, etc)
    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 != 255 {
                    let p2_idx = p2 as usize;
                    let freq = scorer.bigrams_freqs[k];
                    let idx = p1_idx * scorer.key_count + p2_idx;

                    let cost = scorer.full_cost_matrix[idx] * freq;

                    // Distribute cost to both keys involved
                    costs[p1_idx] += cost * 0.5;
                    costs[p2_idx] += cost * 0.5;
                }
            }
        }
    }

    // 3. Trigram Costs
    let k_sq = scorer.key_count * scorer.key_count;
    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.trigram_starts[c1];
        let end = scorer.trigram_starts[c1 + 1];

        for k in start..end {
            let t = &scorer.trigrams_flat[k];
            // Only process role 0 (c1 is first char) to avoid triple counting
            if t.role == 0 {
                let c2 = t.other1 as usize;
                let c3 = t.other2 as usize;
                let p2 = pos_map[c2];
                let p3 = pos_map[c3];
                if p2 != 255 && p3 != 255 {
                    let p2_idx = p2 as usize;
                    let p3_idx = p3 as usize;
                    let idx = p1_idx * k_sq + p2_idx * scorer.key_count + p3_idx;
                    let cost = scorer.trigram_cost_table[idx];
                    if cost != 0.0 {
                        let weighted_cost = cost * t.freq;
                        // Distribute to all 3 keys
                        costs[p1_idx] += weighted_cost * 0.33;
                        costs[p2_idx] += weighted_cost * 0.33;
                        costs[p3_idx] += weighted_cost * 0.33;
                    }
                }
            }
        }
    }

    costs
}
