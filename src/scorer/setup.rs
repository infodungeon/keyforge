use super::flow::analyze_flow;
use super::loader::{load_cost_matrix, load_ngrams, TrigramRef};
use super::physics::{analyze_interaction, get_geo_dist, get_reach_cost};
use super::Scorer;
use crate::config::{LayoutDefinitions, ScoringWeights};
use crate::error::{KeyForgeError, KfResult};
use crate::geometry::KeyboardGeometry;
use std::collections::{HashMap, HashSet};
use std::fs::File;

pub fn build_scorer(
    cost_path: &str,
    ngrams_path: &str,
    weights: ScoringWeights,
    defs: LayoutDefinitions,
    geometry: KeyboardGeometry,
    debug: bool,
) -> KfResult<Scorer> {
    if debug {
        println!("   [Debug] SFB Base: {:.1}", weights.penalty_sfb_base);
    }

    let key_count = geometry.keys.len();
    if key_count == 0 {
        return Err(KeyForgeError::Validation("Geometry has 0 keys".to_string()));
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

    // Load Cost Matrix
    let cost_file = File::open(cost_path).map_err(KeyForgeError::Io)?;
    let costs = load_cost_matrix(cost_file, debug)?;
    let mut raw_user_matrix = vec![0.0; key_count * key_count];

    // Key Mapping
    let mut key_id_map: HashMap<String, usize> = HashMap::new();
    for (idx, k) in geometry.keys.iter().enumerate() {
        if !k.id.is_empty() {
            key_id_map.insert(k.id.to_lowercase(), idx);
        }
    }

    let mut loaded_count = 0;
    for (k1_raw, k2_raw, val) in costs.entries {
        let k1 = k1_raw.to_lowercase();
        let k2 = k2_raw.to_lowercase();

        let i1 = key_id_map.get(&k1);
        let i2 = key_id_map.get(&k2);

        if let (Some(&idx1), Some(&idx2)) = (i1, i2) {
            raw_user_matrix[idx1 * key_count + idx2] = val;
            loaded_count += 1;
        }
    }

    // --- RESTORED LOGIC ---
    if key_id_map.is_empty() && debug {
        println!("   ⚠️  Warning: Geometry keys have no 'id' fields. Cost matrix ignored.");
    } else if loaded_count == 0 && debug {
        println!("   ⚠️  Warning: No costs matched geometry key IDs.");
    }
    // ---------------------

    // Default Fill
    for r in 0..key_count {
        for c in 0..key_count {
            if r != c && raw_user_matrix[r * key_count + c] == 0.0 {
                raw_user_matrix[r * key_count + c] = weights.default_cost_ms;
            }
        }
    }

    let mut full_cost_matrix = raw_user_matrix.clone();

    // Physics Calculation
    for i in 0..key_count {
        for j in 0..key_count {
            if i == j {
                continue;
            }

            let m = analyze_interaction(&geometry, i, j, &weights);

            if m.is_same_hand {
                let dist = get_geo_dist(
                    &geometry,
                    i,
                    j,
                    weights.weight_lateral_travel,
                    weights.weight_vertical_travel,
                );

                let idx = i * key_count + j;
                full_cost_matrix[idx] += dist;

                let res = crate::scorer::costs::calculate_cost(&m, &weights);
                full_cost_matrix[idx] -= res.flow_bonus;
                full_cost_matrix[idx] += res.additive_cost;
                full_cost_matrix[idx] *= res.penalty_multiplier;
            }
        }
    }

    // N-Grams Loading
    let valid_set: HashSet<u8> = b"abcdefghijklmnopqrstuvwxyz.,/;".iter().cloned().collect();
    let ngram_file = File::open(ngrams_path).map_err(KeyForgeError::Io)?;
    let raw_ngrams = load_ngrams(ngram_file, &valid_set, weights.corpus_scale, debug)?;

    if raw_ngrams.bigrams.is_empty() {
        return Err(KeyForgeError::Validation(
            "Ngrams file resulted in 0 valid bigrams.".to_string(),
        ));
    }

    let mut bigram_starts = vec![0; 257];
    let mut bigrams_others = Vec::new();
    let mut bigrams_freqs = Vec::new();
    let mut bigrams_self_first = Vec::new();
    let mut b_buckets: Vec<Vec<(u8, f32, bool)>> = vec![Vec::new(); 256];

    // Flattened Freq Matrix (256 * 256)
    let mut freq_matrix = vec![0.0; 256 * 256];

    for (b1, b2, freq) in raw_ngrams.bigrams {
        b_buckets[b1 as usize].push((b2, freq, true));
        b_buckets[b2 as usize].push((b1, freq, false));
        freq_matrix[(b1 as usize) * 256 + (b2 as usize)] = freq;
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

    // Trigrams
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

    // Trigram Cost Table
    let table_size = key_count * key_count * key_count;
    let mut trigram_cost_table = vec![0.0; table_size];

    for i in 0..key_count {
        for j in 0..key_count {
            for k in 0..key_count {
                let idx = i * (key_count * key_count) + j * key_count + k;
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

    let mut slot_tier_map = vec![0u8; key_count];
    for &i in &geometry.prime_slots {
        if i < key_count {
            slot_tier_map[i] = 0;
        }
    }
    for &i in &geometry.med_slots {
        if i < key_count {
            slot_tier_map[i] = 1;
        }
    }
    for &i in &geometry.low_slots {
        if i < key_count {
            slot_tier_map[i] = 2;
        }
    }

    let mut critical_mask = [false; 256];
    for pair in defs.get_critical_bigrams() {
        critical_mask[pair[0] as usize] = true;
        critical_mask[pair[1] as usize] = true;
    }

    let finger_scales = weights.get_finger_penalty_scale();

    let mut slot_monogram_costs = vec![0.0; key_count];
    for (i, cost) in slot_monogram_costs.iter_mut().enumerate() {
        let ki = &geometry.keys[i];
        let effort_cost = finger_scales[ki.finger as usize] * weights.weight_finger_effort;
        let reach_cost = get_reach_cost(
            &geometry,
            i,
            weights.weight_lateral_travel,
            weights.weight_vertical_travel,
        );
        let stretch_cost = if ki.is_stretch {
            weights.penalty_monogram_stretch
        } else {
            0.0
        };

        *cost = reach_cost + effort_cost + stretch_cost;
    }

    if debug {
        println!(" ✅ Scorer Initialized.\n");
    }

    Ok(Scorer {
        key_count,
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
    })
}
