use crate::consts::{KEY_CODE_RANGE, KEY_NOT_FOUND_U8};
use crate::scorer::flow::analyze_flow;
use crate::scorer::types::MetricViolation;
use crate::scorer::{ScoreDetails, Scorer};

fn fmt_keys(c1: usize, c2: usize, c3: usize) -> String {
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
    let k3 = if (32..=126).contains(&c3) {
        (c3 as u8 as char).to_string()
    } else {
        format!("#{}", c3)
    };
    format!("{} {} {}", k1, k2, k3)
}

#[inline(always)]
pub fn score_trigrams(
    scorer: &Scorer,
    pos_map: &[u8; KEY_CODE_RANGE],
    score: &mut f32,
    limit: usize,
) {
    let kc = scorer.key_count;
    let c_left = scorer.count_left;
    let c_right = scorer.count_right;

    // Unsafe pointers for speed
    let left_ptr = scorer.trigram_left.as_ptr();
    let right_ptr = scorer.trigram_right.as_ptr();
    let hand_ptr = scorer.slot_hand.as_ptr();
    let idx_ptr = scorer.slot_hand_idx.as_ptr();

    for &c1 in &scorer.active_chars {
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
                let p2 = unsafe { *pos_map.get_unchecked(c2) };
                let p3 = unsafe { *pos_map.get_unchecked(c3) };

                if p2 != KEY_NOT_FOUND_U8 && p3 != KEY_NOT_FOUND_U8 {
                    let p2_idx = p2 as usize;
                    let p3_idx = p3 as usize;
                    if p2_idx >= kc || p3_idx >= kc {
                        continue;
                    }

                    unsafe {
                        let h1 = *hand_ptr.add(p1_idx);
                        let h2 = *hand_ptr.add(p2_idx);

                        if h1 == h2 {
                            let h3 = *hand_ptr.add(p3_idx);
                            if h2 == h3 {
                                // All same hand
                                let i1 = *idx_ptr.add(p1_idx);
                                let i2 = *idx_ptr.add(p2_idx);
                                let i3 = *idx_ptr.add(p3_idx);

                                let cost = if h1 == 0 {
                                    *left_ptr.add(i1 * (c_left * c_left) + i2 * c_left + i3)
                                } else {
                                    *right_ptr.add(i1 * (c_right * c_right) + i2 * c_right + i3)
                                };

                                if cost != 0.0 {
                                    *score += cost * t.freq;
                                }
                            }
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
    // Need to replicate the lookup logic here but safe
    let mut redirs = Vec::new();

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

                    let h1 = scorer.slot_hand[p1_idx];
                    let h2 = scorer.slot_hand[p2_idx];
                    let h3 = scorer.slot_hand[p3_idx];

                    if h1 == h2 && h2 == h3 {
                        let i1 = scorer.slot_hand_idx[p1_idx];
                        let i2 = scorer.slot_hand_idx[p2_idx];
                        let i3 = scorer.slot_hand_idx[p3_idx];

                        let cost = if h1 == 0 {
                            let idx = i1 * (scorer.count_left.pow(2)) + i2 * scorer.count_left + i3;
                            scorer.trigram_left[idx]
                        } else {
                            let idx =
                                i1 * (scorer.count_right.pow(2)) + i2 * scorer.count_right + i3;
                            scorer.trigram_right[idx]
                        };

                        if cost != 0.0 {
                            d.flow_cost += cost * t.freq;
                            let k1 = &scorer.geometry.keys[p1_idx];
                            let k2 = &scorer.geometry.keys[p2_idx];
                            let k3 = &scorer.geometry.keys[p3_idx];
                            let flow = analyze_flow(k1, k2, k3);

                            if flow.is_3_hand_run {
                                if flow.is_redirect {
                                    d.stat_redir += t.freq;
                                    let pen = scorer.weights.penalty_redirect * t.freq;
                                    d.flow_redirect += pen;
                                    redirs.push(MetricViolation {
                                        keys: fmt_keys(c1, c2, c3),
                                        score: pen,
                                        freq: t.freq,
                                    });
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

    redirs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    redirs.truncate(10);
    d.top_redirs = redirs;
}

pub fn accumulate_key_costs(scorer: &Scorer, pos_map: &[u8; KEY_CODE_RANGE], costs: &mut [f32]) {
    // Similar logic: check hands, look up cost
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

                    let h1 = scorer.slot_hand[p1_idx];
                    let h2 = scorer.slot_hand[p2_idx];
                    let h3 = scorer.slot_hand[p3_idx];

                    if h1 == h2 && h2 == h3 {
                        let i1 = scorer.slot_hand_idx[p1_idx];
                        let i2 = scorer.slot_hand_idx[p2_idx];
                        let i3 = scorer.slot_hand_idx[p3_idx];

                        let cost = if h1 == 0 {
                            scorer.trigram_left
                                [i1 * (scorer.count_left.pow(2)) + i2 * scorer.count_left + i3]
                        } else {
                            scorer.trigram_right
                                [i1 * (scorer.count_right.pow(2)) + i2 * scorer.count_right + i3]
                        };

                        if cost != 0.0 {
                            let weighted_cost = cost * t.freq;
                            costs[p1_idx] += weighted_cost * 0.33;
                            costs[p2_idx] += weighted_cost * 0.33;
                            costs[p3_idx] += weighted_cost * 0.33;
                        }
                    }
                }
            }
        }
    }
}
