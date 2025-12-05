use crate::config::{LayoutDefinitions, ScoringWeights};
use crate::error::KfResult;
use crate::geometry::KeyboardGeometry;
use crate::scorer::loader::{
    load_corpus_bundle, load_cost_matrix, CorpusBundle, RawCostData, TrigramRef,
};
use crate::scorer::Scorer;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::debug;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct ScorerBuildParams {
    #[builder(default)]
    pub weights: ScoringWeights,
    #[builder(default)]
    pub defs: LayoutDefinitions,
    pub geometry: KeyboardGeometry,
    pub cost_data: RawCostData,
    pub corpus: CorpusBundle,
    #[builder(default = false)]
    pub debug: bool,
}

impl ScorerBuildParams {
    pub fn load_from_disk<P1: AsRef<Path>, P2: AsRef<Path>>(
        cost_path: P1,
        corpus_dir: P2,
        geometry: KeyboardGeometry,
        weights: Option<ScoringWeights>,
        defs: Option<LayoutDefinitions>,
        debug: bool,
    ) -> KfResult<Scorer> {
        let final_weights = weights.unwrap_or_default();

        let cost_data = load_cost_matrix(cost_path)?;
        let corpus = load_corpus_bundle(
            corpus_dir,
            final_weights.corpus_scale,
            final_weights.loader_trigram_limit,
        )?;

        ScorerBuildParams::builder()
            .weights(final_weights)
            .defs(defs.unwrap_or_default())
            .geometry(geometry)
            .cost_data(cost_data)
            .corpus(corpus)
            .debug(debug)
            .build()
            .build_scorer()
    }

    pub fn build_scorer(self) -> KfResult<Scorer> {
        let weights = self.weights;
        let defs = self.defs;
        let geometry = self.geometry;
        let cost_data = self.cost_data;
        let corpus = self.corpus;
        let debug = self.debug;

        let key_count = geometry.keys.len();

        // 1. Build Cost Matrix
        let mut raw_user_matrix = vec![0.0; key_count * key_count];
        let mut key_id_map: HashMap<String, usize> = HashMap::new();
        for (idx, k) in geometry.keys.iter().enumerate() {
            if !k.id.is_empty() {
                key_id_map.insert(k.id.to_lowercase(), idx);
            }
        }

        for (k1_raw, k2_raw, val) in cost_data.entries {
            let k1 = k1_raw.to_lowercase();
            let k2 = k2_raw.to_lowercase();
            if let (Some(&idx1), Some(&idx2)) = (key_id_map.get(&k1), key_id_map.get(&k2)) {
                raw_user_matrix[idx1 * key_count + idx2] = val;
            }
        }

        for r in 0..key_count {
            for c in 0..key_count {
                if r != c && raw_user_matrix[r * key_count + c] == 0.0 {
                    raw_user_matrix[r * key_count + c] = weights.default_cost_ms;
                }
            }
        }

        let mut full_cost_matrix = raw_user_matrix.clone();

        for i in 0..key_count {
            for j in 0..key_count {
                if i == j {
                    continue;
                }
                let m = crate::scorer::physics::analyze_interaction(&geometry, i, j, &weights);
                if m.is_same_hand {
                    let dist = crate::scorer::physics::get_geo_dist(
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

        // 2. Build N-Gram Lookups
        let mut bigram_starts = vec![0; 257];
        let mut bigrams_others = Vec::new();
        let mut bigrams_freqs = Vec::new();
        let mut bigrams_self_first = Vec::new();
        let mut b_buckets: Vec<Vec<(u8, f32, bool)>> = vec![Vec::new(); 256];
        let mut freq_matrix = vec![0.0; 256 * 256];
        let mut active_chars_set: HashSet<usize> = HashSet::new();

        for (b1, b2, freq) in corpus.bigrams {
            b_buckets[b1 as usize].push((b2, freq, true));
            b_buckets[b2 as usize].push((b1, freq, false));
            freq_matrix[(b1 as usize) * 256 + (b2 as usize)] = freq;
            active_chars_set.insert(b1 as usize);
            active_chars_set.insert(b2 as usize);
        }

        for (i, &f) in corpus.char_freqs.iter().enumerate() {
            if f > 0.0 {
                active_chars_set.insert(i);
            }
        }

        let mut active_chars: Vec<usize> = active_chars_set.into_iter().collect();
        active_chars.sort();

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
        for (b1, b2, b3, freq) in corpus.trigrams {
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

        // 3. OPTIMIZED Trigram Physics Cost Table
        // Split keys into Left and Right sets
        let mut slot_hand = vec![0u8; key_count];
        let mut slot_hand_idx = vec![0usize; key_count];
        let mut count_left = 0;
        let mut count_right = 0;

        for (i, k) in geometry.keys.iter().enumerate() {
            if k.hand == 0 {
                slot_hand[i] = 0;
                slot_hand_idx[i] = count_left;
                count_left += 1;
            } else {
                slot_hand[i] = 1;
                slot_hand_idx[i] = count_right;
                count_right += 1;
            }
        }

        let size_left = count_left * count_left * count_left;
        let size_right = count_right * count_right * count_right;

        let mut trigram_left = vec![0.0; size_left];
        let mut trigram_right = vec![0.0; size_right];

        for i in 0..key_count {
            for j in 0..key_count {
                for k in 0..key_count {
                    let h1 = slot_hand[i];
                    let h2 = slot_hand[j];
                    let h3 = slot_hand[k];

                    // Only calculate if all on same hand
                    if h1 == h2 && h2 == h3 {
                        let ki = &geometry.keys[i];
                        let kj = &geometry.keys[j];
                        let kk = &geometry.keys[k];
                        let flow = crate::scorer::flow::analyze_flow(ki, kj, kk);

                        let mut cost = 0.0;
                        if flow.is_3_hand_run {
                            cost += weights.penalty_hand_run;
                            if flow.is_skip {
                                cost += weights.penalty_skip;
                            }
                            if flow.is_redirect {
                                cost += weights.penalty_redirect;
                            }
                            if flow.is_inward_roll {
                                cost -= weights.bonus_inward_roll;
                            }
                        }

                        if cost != 0.0 {
                            let idx1 = slot_hand_idx[i];
                            let idx2 = slot_hand_idx[j];
                            let idx3 = slot_hand_idx[k];

                            if h1 == 0 {
                                let flat_idx =
                                    idx1 * (count_left * count_left) + idx2 * count_left + idx3;
                                trigram_left[flat_idx] = cost;
                            } else {
                                let flat_idx =
                                    idx1 * (count_right * count_right) + idx2 * count_right + idx3;
                                trigram_right[flat_idx] = cost;
                            }
                        }
                    }
                }
            }
        }

        // 4. Lookup Maps
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
            let reach_cost = crate::scorer::physics::get_reach_cost(
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
            debug!(
                "✅ Scorer Initialized. Active Chars: {}",
                active_chars.len()
            );
            debug!(
                "   L-Table: {} entries, R-Table: {} entries",
                size_left, size_right
            );
        }

        Ok(Scorer {
            key_count,
            weights,
            defs,
            geometry,
            tier_penalty_matrix,
            full_cost_matrix,
            raw_user_matrix,

            // New Fields
            trigram_left,
            trigram_right,
            count_left,
            count_right,
            slot_hand,
            slot_hand_idx,

            bigram_starts,
            bigrams_others,
            bigrams_freqs,
            bigrams_self_first,
            trigram_starts,
            trigrams_flat,
            char_freqs: corpus.char_freqs,
            words: corpus.words,
            char_tier_map,
            slot_tier_map,
            critical_mask,
            freq_matrix,
            finger_scales,
            slot_monogram_costs,
            active_chars,
        })
    }
}
