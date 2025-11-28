use super::flow::analyze_flow;
use super::loader::{load_cost_matrix, load_ngrams, TrigramRef};
use super::physics::{analyze_interaction, get_geo_dist, get_reach_cost};
use super::Scorer;
use crate::config::{LayoutDefinitions, ScoringWeights};
use crate::geometry::KeyboardGeometry;
use std::collections::HashSet;

pub fn build_scorer(
    cost_path: &str,
    ngrams_path: &str,
    weights: ScoringWeights,
    defs: LayoutDefinitions,
    geometry: KeyboardGeometry,
    debug: bool,
) -> Scorer {
    if debug {
        println!(
            "\n   [Debug] SFB Base: {:.1} | Long: {:.1} | Lat: {:.1} | Weak Lat: {:.1}",
            weights.penalty_sfb_base,
            weights.penalty_sfb_long,
            weights.penalty_sfb_lateral,
            weights.penalty_sfb_lateral_weak
        );
        println!(
            "   [Debug] Bigram Rolls: In -{:.1} | Out -{:.1}",
            weights.bonus_bigram_roll_in, weights.bonus_bigram_roll_out
        );
    }

    let tier_penalty_matrix = [
        [
            0.0,
            weights.penalty_high_in_med,
            weights.penalty_high_in_low,
        ],
        [
            weights.penalty_med_in_prime,
            0.0,
            weights.penalty_med_in_low,
        ],
        [
            weights.penalty_low_in_prime,
            weights.penalty_low_in_med,
            0.0,
        ],
    ];

    let costs = load_cost_matrix(cost_path, debug);
    let mut raw_user_matrix = [[0.0; 30]; 30];

    // FILLER LOGIC START (Ensures valid matrix even with sparse CSV)
    let standard_keys = [
        "keyq",
        "keyw",
        "keye",
        "keyr",
        "keyt",
        "keyy",
        "keyu",
        "keyi",
        "keyo",
        "keyp",
        "keya",
        "keys",
        "keyd",
        "keyf",
        "keyg",
        "keyh",
        "keyj",
        "keyk",
        "keyl",
        "semicolon",
        "keyz",
        "keyx",
        "keyc",
        "keyv",
        "keyb",
        "keyn",
        "keym",
        "comma",
        "period",
        "slash",
    ];
    let mut loaded_count = 0;
    for (k1_raw, k2_raw, val) in costs.entries {
        let k1 = k1_raw.to_lowercase();
        let k2 = k2_raw.to_lowercase();
        let find = |k: &str| {
            standard_keys
                .iter()
                .position(|&sk| sk == k || sk.strip_prefix("key").unwrap_or("") == k)
        };
        if let (Some(i1), Some(i2)) = (find(&k1), find(&k2)) {
            raw_user_matrix[i1][i2] = val;
            loaded_count += 1;
        }
    }

    // FIX: Use iterator to fill missing values (needless_range_loop)
    if loaded_count < 10 {
        for (r, row) in raw_user_matrix.iter_mut().enumerate() {
            for (col, val) in row.iter_mut().enumerate() {
                if r != col {
                    *val = weights.default_cost_ms;
                }
            }
        }
    }
    // FILLER LOGIC END

    let mut full_cost_matrix = raw_user_matrix;

    // FIX: Use iterator for matrix construction (needless_range_loop)
    for (i, row) in full_cost_matrix.iter_mut().enumerate() {
        for (j, val) in row.iter_mut().enumerate() {
            if i == j {
                continue;
            }

            let m = analyze_interaction(&geometry, i, j);
            if m.is_same_hand {
                // Add Physical Distance to Base Cost
                let dist = get_geo_dist(&geometry, i, j, weights.weight_geo_dist);
                *val += dist;

                // === BIGRAM FLOW BONUSES ===
                // Rewards rolling motion between different fingers
                if m.is_roll_in {
                    *val -= weights.bonus_bigram_roll_in;
                } else if m.is_roll_out {
                    *val -= weights.bonus_bigram_roll_out;
                }

                // === BIOMECHANICAL PENALTIES ===
                if m.is_repeat {
                    // SFR (Same Finger Repeat)
                    if m.is_strong_finger {
                        if m.is_home_row {
                            // Rank 1: Strong Home SFR (TT) -> 0.0 (No Penalty)
                            *val *= 1.0;
                        } else if m.is_stretch_col {
                            // Rank 5: Strong Lat SFR (GG)
                            *val += weights.penalty_sfr_lat;
                        } else {
                            // Rank 3: Strong Bad Row SFR (PP)
                            *val += weights.penalty_sfr_bad_row;
                        }
                    } else if m.is_home_row {
                        // FIX: Collapsed else { if } (collapsible_else_if)
                        // Rank 2: Weak Home SFR (RR)
                        *val += weights.penalty_sfr_weak_finger;
                    } else {
                        // Rank 11: Weak Bad Row SFR (ZZ) -> Massive Penalty
                        *val += weights.penalty_sfr_bad_row * 5.0;
                    }
                } else if m.is_sfb {
                    // SFB (Same Finger Bigram)
                    let mut penalty;
                    let mut weak_applied = false;

                    if m.is_lat_step {
                        if m.is_strong_finger {
                            // Rank 4: Index Lateral SFB (TG)
                            penalty = weights.penalty_sfb_lateral;
                        } else {
                            // Rank 10+: Weak Lateral SFB
                            penalty = weights.penalty_sfb_lateral_weak;
                            weak_applied = true;
                        }
                    } else if m.is_bot_lat_seq {
                        // Rank 9: Bot Lat (The Claw)
                        penalty = weights.penalty_sfb_bottom;
                    } else if m.row_diff >= 2 {
                        // Rank 8: Long Jump
                        penalty = weights.penalty_sfb_long;
                    } else if m.row_diff > 0 && m.col_diff > 0 {
                        // Rank 7: Diagonal
                        penalty = weights.penalty_sfb_diagonal;
                    } else {
                        // Rank 6: Standard 1-Row
                        penalty = weights.penalty_sfb_base;
                        if m.is_outward {
                            penalty += weights.penalty_sfb_outward_adder;
                        }
                    }

                    // Apply Weak Finger Multiplier if not already handled
                    if !m.is_strong_finger && !weak_applied {
                        penalty *= weights.weight_weak_finger_sfb;
                    }

                    *val *= penalty;
                } else if m.is_scissor {
                    *val *= weights.penalty_scissor;
                } else if m.is_lateral_stretch {
                    *val *= weights.penalty_lateral;
                }
            }
        }
    }

    let valid_set: HashSet<u8> = b"abcdefghijklmnopqrstuvwxyz.,/;".iter().cloned().collect();
    let raw_ngrams = load_ngrams(ngrams_path, &valid_set, weights.corpus_scale, debug);
    if raw_ngrams.bigrams.is_empty() {
        panic!("FATAL: Ngrams empty");
    }

    let mut bigram_starts = vec![0; 257];
    let mut bigrams_others = Vec::new();
    let mut bigrams_freqs = Vec::new();
    let mut bigrams_self_first = Vec::new();
    let mut b_buckets: Vec<Vec<(u8, f32, bool)>> = vec![Vec::new(); 256];
    let mut freq_matrix = [[0.0; 256]; 256];

    for (b1, b2, freq) in raw_ngrams.bigrams {
        b_buckets[b1 as usize].push((b2, freq, true));
        b_buckets[b2 as usize].push((b1, freq, false));
        freq_matrix[b1 as usize][b2 as usize] = freq;
    }

    let mut offset = 0;
    for i in 0..256 {
        bigram_starts[i] = offset;
        for (other, freq, is_first) in &b_buckets[i] {
            bigrams_others.push(*other);
            bigrams_freqs.push(*freq);
            bigrams_self_first.push(*is_first);
        }
        offset += b_buckets[i].len();
    }
    bigram_starts[256] = offset;

    let mut t_buckets: Vec<Vec<TrigramRef>> = vec![Vec::new(); 256];
    for (b1, b2, b3, freq) in raw_ngrams.trigrams {
        t_buckets[b1 as usize].push(TrigramRef {
            other1: b2,
            other2: b3,
            freq,
            role: 0,
        });
        t_buckets[b2 as usize].push(TrigramRef {
            other1: b1,
            other2: b3,
            freq,
            role: 1,
        });
        t_buckets[b3 as usize].push(TrigramRef {
            other1: b1,
            other2: b2,
            freq,
            role: 2,
        });
    }

    let mut trigram_starts = vec![0; 257];
    let mut trigrams_flat = Vec::new();
    let mut t_offset = 0;
    for i in 0..256 {
        trigram_starts[i] = t_offset;
        for t_ref in &t_buckets[i] {
            trigrams_flat.push(t_ref.clone());
        }
        t_offset += t_buckets[i].len();
    }
    trigram_starts[256] = t_offset;

    let mut trigram_cost_table = vec![0.0; 27000];
    for i in 0..30 {
        for j in 0..30 {
            for k in 0..30 {
                let idx = i * 900 + j * 30 + k;
                let ki = &geometry.keys[i];
                let kj = &geometry.keys[j];
                let kk = &geometry.keys[k];

                let flow = analyze_flow(ki, kj, kk);
                if flow.is_3_hand_run {
                    trigram_cost_table[idx] += weights.penalty_hand_run;
                    if flow.is_skip {
                        trigram_cost_table[idx] += weights.penalty_skip;
                    }
                    if flow.is_redirect {
                        trigram_cost_table[idx] += weights.penalty_redirect;
                    }
                    if flow.is_inward_roll {
                        trigram_cost_table[idx] -= weights.bonus_inward_roll;
                    }
                }
            }
        }
    }

    let mut char_tier_map = [2u8; 256];
    for b in defs.tier_high_chars.bytes() {
        char_tier_map[b as usize] = 0;
    }
    for b in defs.tier_med_chars.bytes() {
        char_tier_map[b as usize] = 1;
    }

    let mut slot_tier_map = [0u8; 30];
    for &i in &geometry.prime_slots {
        slot_tier_map[i] = 0;
    }
    for &i in &geometry.med_slots {
        slot_tier_map[i] = 1;
    }
    for &i in &geometry.low_slots {
        slot_tier_map[i] = 2;
    }

    let mut critical_mask = [false; 256];
    for pair in defs.get_critical_bigrams() {
        critical_mask[pair[0] as usize] = true;
        critical_mask[pair[1] as usize] = true;
    }

    let finger_scales = weights.get_finger_penalty_scale();
    let mut slot_monogram_costs = [0.0; 30];

    // FIX: Use iterator for monogram costs (needless_range_loop)
    for (i, cost) in slot_monogram_costs.iter_mut().enumerate() {
        let ki = &geometry.keys[i];
        let effort_cost = finger_scales[ki.finger as usize] * weights.weight_finger_effort;
        let reach_cost = get_reach_cost(&geometry, i, weights.weight_geo_dist);
        *cost = reach_cost + effort_cost;
    }

    if debug {
        println!(" ✅ Scorer Initialized.\n");
    }

    Scorer {
        weights,
        defs,
        geometry,
        tier_penalty_matrix,
        full_cost_matrix,
        raw_user_matrix,
        trigram_cost_table,
        bigram_starts,
        bigrams_others,
        bigrams_freqs,
        bigrams_self_first,
        trigram_starts,
        trigrams_flat,
        char_freqs: raw_ngrams.char_freqs,
        char_tier_map,
        slot_tier_map,
        critical_mask,
        freq_matrix,
        finger_scales,
        slot_monogram_costs,
    }
}
