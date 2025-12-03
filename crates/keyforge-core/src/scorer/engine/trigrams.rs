use crate::consts::{KEY_CODE_RANGE, KEY_NOT_FOUND_U8};
use crate::scorer::flow::analyze_flow;
use crate::scorer::{ScoreDetails, Scorer};

#[inline(always)]
pub fn score_trigrams(
    scorer: &Scorer,
    pos_map: &[u8; KEY_CODE_RANGE],
    score: &mut f32,
    limit: usize,
) {
    let k_sq = scorer.key_count * scorer.key_count;
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

        let start = unsafe { *scorer.trigram_starts.get_unchecked(c1) };
        let end = unsafe { *scorer.trigram_starts.get_unchecked(c1 + 1) };

        let len = end - start;
        let effective_len = if len > limit { limit } else { len };
        let effective_end = start + effective_len;

        for k in start..effective_end {
            let t = unsafe { scorer.trigrams_flat.get_unchecked(k) };

            if t.role == 0 {
                let c2 = t.other1 as usize;
                let c3 = t.other2 as usize;
                debug_assert!(c2 < KEY_CODE_RANGE);
                debug_assert!(c3 < KEY_CODE_RANGE);

                let p2 = unsafe { *pos_map.get_unchecked(c2) };
                let p3 = unsafe { *pos_map.get_unchecked(c3) };

                if p2 != KEY_NOT_FOUND_U8 && p3 != KEY_NOT_FOUND_U8 {
                    let p2_idx = p2 as usize;
                    let p3_idx = p3 as usize;
                    if p2_idx >= kc || p3_idx >= kc {
                        continue;
                    }

                    let idx = p1_idx * k_sq + p2_idx * kc + p3_idx;
                    debug_assert!(idx < scorer.trigram_cost_table.len());

                    unsafe {
                        let cost = *scorer.trigram_cost_table.get_unchecked(idx);
                        if cost != 0.0 {
                            *score += cost * t.freq;
                        }
                    }
                }
            }
        }
    }
}

pub fn accumulate_details(
    scorer: &Scorer,
    pos_map: &[u8; KEY_CODE_RANGE],
    d: &mut ScoreDetails,
    limit: usize,
) {
    let k_sq = scorer.key_count * scorer.key_count;

    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == KEY_NOT_FOUND_U8 {
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

                if p2 != KEY_NOT_FOUND_U8 && p3 != KEY_NOT_FOUND_U8 {
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
}

pub fn accumulate_key_costs(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], costs: &mut [f32]) {
    let k_sq = scorer.key_count * scorer.key_count;

    for &c1 in &scorer.active_chars {
        let p1 = pos_map[c1];
        if p1 == KEY_NOT_FOUND_U8 {
            continue;
        }
        let p1_idx = p1 as usize;

        let start = scorer.trigram_starts[c1];
        let end = scorer.trigram_starts[c1 + 1];

        for k in start..end {
            let t = &scorer.trigrams_flat[k];
            if t.role == 0 {
                let c2 = t.other1 as usize;
                let c3 = t.other2 as usize;
                let p2 = pos_map[c2];
                let p3 = pos_map[c3];

                if p2 != KEY_NOT_FOUND_U8 && p3 != KEY_NOT_FOUND_U8 {
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
}
