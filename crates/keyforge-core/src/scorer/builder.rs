use crate::config::{LayoutDefinitions, ScoringWeights};
use crate::error::{KeyForgeError, KfResult};
use crate::geometry::KeyboardGeometry;
use crate::scorer::flow::analyze_flow;
use crate::scorer::loader::{load_cost_matrix, load_ngrams, RawCostData, RawNgrams, TrigramRef};
use crate::scorer::physics::{analyze_interaction, get_geo_dist, get_reach_cost};
use crate::scorer::Scorer;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::{debug, warn};

pub struct ScorerBuilder {
    weights: Option<ScoringWeights>,
    defs: Option<LayoutDefinitions>,
    geometry: Option<KeyboardGeometry>,
    cost_data: Option<RawCostData>,
    ngram_data: Option<RawNgrams>,
    debug: bool,
}

impl Default for ScorerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScorerBuilder {
    pub fn new() -> Self {
        Self {
            weights: None,
            defs: None,
            geometry: None,
            cost_data: None,
            ngram_data: None,
            debug: false,
        }
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_weights(mut self, weights: ScoringWeights) -> Self {
        self.weights = Some(weights);
        self
    }

    pub fn with_defs(mut self, defs: LayoutDefinitions) -> Self {
        self.defs = Some(defs);
        self
    }

    pub fn with_geometry(mut self, geometry: KeyboardGeometry) -> Self {
        self.geometry = Some(geometry);
        self
    }

    pub fn with_costs_from_reader<R: Read>(mut self, reader: R) -> KfResult<Self> {
        let data = load_cost_matrix(reader, self.debug)?;
        self.cost_data = Some(data);
        Ok(self)
    }

    pub fn with_costs_from_file<P: AsRef<Path>>(self, path: P) -> KfResult<Self> {
        let file = File::open(path).map_err(KeyForgeError::Io)?;
        self.with_costs_from_reader(file)
    }

    pub fn with_ngrams_from_reader<R: Read>(mut self, reader: R) -> KfResult<Self> {
        // Resolve scale and limit from weights if present, otherwise defaults
        let (scale, limit) = if let Some(w) = &self.weights {
            (w.corpus_scale, w.loader_trigram_limit)
        } else {
            (200_000_000.0, 3000)
        };

        // FIXED: Expanded valid set to include apostrophe, hyphen, and other common punctuation
        // to prevent dropping valid English n-grams like "don't" or "it's".
        let valid_set: HashSet<u8> = b"abcdefghijklmnopqrstuvwxyz.,/;'[]-!?:\"()"
            .iter()
            .cloned()
            .collect();

        let data = load_ngrams(reader, &valid_set, scale, limit, self.debug)?;

        // Debug check to ensure we actually loaded data.
        if self.debug && data.bigrams.is_empty() {
            warn!("⚠️ Warning: 0 bigrams loaded. Check your N-gram file format or encoding.");
        }

        self.ngram_data = Some(data);
        Ok(self)
    }

    pub fn with_ngrams_from_file<P: AsRef<Path>>(self, path: P) -> KfResult<Self> {
        let file = File::open(path).map_err(KeyForgeError::Io)?;
        self.with_ngrams_from_reader(file)
    }

    pub fn build(self) -> KfResult<Scorer> {
        let weights = self.weights.unwrap_or_default();
        let defs = self.defs.unwrap_or_default();
        let geometry = self
            .geometry
            .ok_or_else(|| KeyForgeError::Validation("Geometry is required".into()))?;
        let cost_data = self.cost_data.unwrap_or(RawCostData { entries: vec![] });
        let raw_ngrams = self
            .ngram_data
            .ok_or_else(|| KeyForgeError::Validation("N-gram data is required".into()))?;

        if self.debug {
            debug!("SFB Base Penalty: {:.1}", weights.penalty_sfb_base);
        }

        let key_count = geometry.keys.len();
        if key_count == 0 {
            return Err(KeyForgeError::Validation("Geometry has 0 keys".to_string()));
        }

        // --- Build Tier Matrix ---
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

        // --- Build User Cost Matrix ---
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

        // Fill defaults for missing user costs
        for r in 0..key_count {
            for c in 0..key_count {
                if r != c && raw_user_matrix[r * key_count + c] == 0.0 {
                    raw_user_matrix[r * key_count + c] = weights.default_cost_ms;
                }
            }
        }

        let mut full_cost_matrix = raw_user_matrix.clone();

        // --- Physics Calculation ---
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

        // --- N-Gram Processing & Optimization ---
        let mut bigram_starts = vec![0; 257];
        let mut bigrams_others = Vec::new();
        let mut bigrams_freqs = Vec::new();
        let mut bigrams_self_first = Vec::new();
        let mut b_buckets: Vec<Vec<(u8, f32, bool)>> = vec![Vec::new(); 256];
        let mut freq_matrix = vec![0.0; 256 * 256];

        // Track which characters actually appear in the corpus to optimize hot loops
        let mut active_chars_set: HashSet<usize> = HashSet::new();

        for (b1, b2, freq) in raw_ngrams.bigrams {
            b_buckets[b1 as usize].push((b2, freq, true));
            b_buckets[b2 as usize].push((b1, freq, false));
            freq_matrix[(b1 as usize) * 256 + (b2 as usize)] = freq;

            active_chars_set.insert(b1 as usize);
            active_chars_set.insert(b2 as usize);
        }

        // Also check monograms
        for (i, &f) in raw_ngrams.char_freqs.iter().enumerate() {
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

        // --- Trigram Processing ---
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

        // --- Trigram Cost Table ---
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

        // --- Static Maps ---
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

        // === NEW: Stability Check ===
        // Ensure no NaNs or Infinities exist in the matrices
        for (i, &val) in full_cost_matrix.iter().enumerate() {
            if !val.is_finite() {
                return Err(KeyForgeError::Validation(format!(
                    "Cost Matrix contains non-finite value at index {}: {}",
                    i, val
                )));
            }
        }

        for (i, &val) in trigram_cost_table.iter().enumerate() {
            if !val.is_finite() {
                return Err(KeyForgeError::Validation(format!(
                    "Trigram Cost Table contains non-finite value at index {}: {}",
                    i, val
                )));
            }
        }

        if self.debug {
            debug!(
                "✅ Scorer Initialized. Active Chars: {}",
                active_chars.len()
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
            active_chars,
        })
    }
}
