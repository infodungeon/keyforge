// ===== keyforge/crates/keyforge-core/src/scorer/engine/bigrams.rs =====
use crate::consts::{KEY_CODE_RANGE, KEY_NOT_FOUND_U8};
use crate::scorer::costs::{calculate_cost, CostCategory};
use crate::scorer::physics::{analyze_interaction, get_geo_dist};
use crate::scorer::types::MetricViolation;
use crate::scorer::{ScoreDetails, Scorer};

// Helper to format key labels roughly
fn fmt_keys(c1: usize, c2: usize) -> String {
    // FIXED: Use range syntax
    let k1 = if (32..=126).contains(&c1) {
        (c1 as u8 as char).to_string()
    } else {
        format!("#{}", c1)
    };

    let k2 = if (32..=126).contains(&c2) {
        (c2 as u8 as char).to_string()
    } else {
        format!("#{}", c2)
    };

    format!("{} {}", k1, k2)
}

#[inline(always)]
pub fn score_bigrams(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], score: &mut f32) {
    let kc = scorer.key_count;
    for &c1 in &scorer.active_chars {
        debug_assert!(c1 < KEY_CODE_RANGE);
        let p1 = unsafe { *pos_map.get_unchecked(c1) };
        if p1 == KEY_NOT_FOUND_U8 {
            continue;
        }
        let p1_idx = p1 as usize;
        if p1_idx >= kc {
            continue;
        }

        let start = unsafe { *scorer.bigram_starts.get_unchecked(c1) };
        let end = unsafe { *scorer.bigram_starts.get_unchecked(c1 + 1) };

        for k in start..end {
            if unsafe { *scorer.bigrams_self_first.get_unchecked(k) } {
                let c2 = unsafe { *scorer.bigrams_others.get_unchecked(k) } as usize;
                debug_assert!(c2 < KEY_CODE_RANGE);
                let p2 = unsafe { *pos_map.get_unchecked(c2) };

                if p2 != KEY_NOT_FOUND_U8 {
                    let p2_idx = p2 as usize;
                    if p2_idx >= kc {
                        continue;
                    }
                    let idx = p1_idx * kc + p2_idx;
                    unsafe {
                        *score += *scorer.full_cost_matrix.get_unchecked(idx)
                            * *scorer.bigrams_freqs.get_unchecked(k);
                    }
                }
            }
        }
    }
}

pub fn accumulate_details(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], d: &mut ScoreDetails) {
    let mut sfbs = Vec::new();
    let mut scissors = Vec::new();

    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == KEY_NOT_FOUND_U8 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];

                if p2 != KEY_NOT_FOUND_U8 {
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
                            let res = calculate_cost(&m, &scorer.weights);
                            let cost = (dist * res.penalty_multiplier) * freq;
                            sfbs.push(MetricViolation {
                                keys: fmt_keys(c1, c2),
                                score: cost,
                                freq,
                            });
                        }

                        if m.is_scissor {
                            d.stat_scis += freq;
                            let res = calculate_cost(&m, &scorer.weights);
                            let cost = (dist * res.penalty_multiplier) * freq;
                            scissors.push(MetricViolation {
                                keys: fmt_keys(c1, c2),
                                score: cost,
                                freq,
                            });
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

    sfbs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    sfbs.truncate(10);
    d.top_sfbs = sfbs;

    scissors.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    scissors.truncate(10);
    d.top_scissors = scissors;
}

pub fn accumulate_key_costs(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], costs: &mut [f32]) {
    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == KEY_NOT_FOUND_U8 {
            continue;
        }
        let p1_idx = p1 as usize;
        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];
        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 != KEY_NOT_FOUND_U8 {
                    let p2_idx = p2 as usize;
                    let freq = scorer.bigrams_freqs[k];
                    let idx = p1_idx * scorer.key_count + p2_idx;
                    let cost = scorer.full_cost_matrix[idx] * freq;
                    costs[p1_idx] += cost * 0.5;
                    costs[p2_idx] += cost * 0.5;
                }
            }
        }
    }
}
