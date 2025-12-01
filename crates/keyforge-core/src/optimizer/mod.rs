pub mod crossover;
pub mod mutation;
pub mod runner;

use self::mutation::*;
pub use self::runner::{OptimizationOptions, Optimizer, ProgressCallback};

use crate::scorer::Scorer;
use itertools::Itertools;
use std::sync::Arc;

#[inline(always)]
fn fast_exp(x: f32) -> f32 {
    let x = 1.0 + x / 256.0;
    let x = x * x * x * x * x * x * x * x;
    x * x
}

#[repr(align(64))]
pub struct Replica {
    pub scorer: Arc<Scorer>,
    pub local_cost_matrix: Vec<f32>,
    pub local_trigram_costs: Vec<f32>,
    pub local_monogram_costs: Vec<f32>,
    pub layout: Vec<u8>,
    pub pos_map: [u8; 256],
    pub score: f32,
    pub left_load: f32,
    pub total_freq: f32,
    pub temperature: f32,
    pub debug: bool,
    pub current_limit: usize,
    pub limit_fast: usize,
    pub limit_slow: usize,
    pub rng: fastrand::Rng,
    pub pinned_slots: Vec<Option<u8>>,
    pub locked_indices: Vec<usize>,

    // Weighted Mutation Fields
    pub mutation_weights: Vec<f32>,
    pub total_weight: f32,
}

impl Replica {
    pub fn new(
        scorer: Arc<Scorer>,
        temperature: f32,
        seed: Option<u64>,
        debug: bool,
        limit_fast: usize,
        limit_slow: usize,
        pinned_keys_str: &str,
    ) -> Self {
        let mut rng = if let Some(s) = seed {
            fastrand::Rng::with_seed(s)
        } else {
            fastrand::Rng::new()
        };

        let key_count = scorer.key_count;

        let mut pinned_slots = vec![None; key_count];
        let mut locked_indices = Vec::new();

        if !pinned_keys_str.is_empty() {
            for part in pinned_keys_str.split(',') {
                let parts: Vec<&str> = part.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(idx) = parts[0].trim().parse::<usize>() {
                        if idx < key_count {
                            let char_str = parts[1].trim();
                            if let Some(c) = char_str.chars().next() {
                                let byte = c.to_ascii_lowercase() as u8;
                                pinned_slots[idx] = Some(byte);
                                locked_indices.push(idx);
                            }
                        }
                    }
                }
            }
        }

        locked_indices.sort();

        let mut layout;
        let mut pos_map;

        loop {
            layout = mutation::generate_tiered_layout(
                &mut rng,
                &scorer.defs,
                &scorer.geometry,
                key_count,
                &pinned_slots,
            );
            pos_map = mutation::build_pos_map(&layout);

            let critical = scorer.defs.get_critical_bigrams();
            if !mutation::fails_sanity(&pos_map, &critical, &scorer.geometry) {
                break;
            }
        }

        let start_limit = if temperature > 10.0 {
            limit_fast
        } else {
            limit_slow
        };

        let (base, left, total) = scorer.score_full(&pos_map, start_limit);

        let local_cost_matrix = scorer.full_cost_matrix.clone();
        let local_trigram_costs = scorer.trigram_cost_table.clone();
        let local_monogram_costs = scorer.slot_monogram_costs.clone();

        let mut r = Replica {
            scorer,
            local_cost_matrix,
            local_trigram_costs,
            local_monogram_costs,
            layout,
            pos_map,
            score: base,
            left_load: left,
            total_freq: total,
            temperature,
            debug,
            current_limit: start_limit,
            limit_fast,
            limit_slow,
            rng,
            pinned_slots,
            locked_indices,
            mutation_weights: vec![1.0; key_count], // Init weights
            total_weight: key_count as f32,
        };

        let imb = r.imbalance_penalty(left);
        r.score += imb;

        // Initial Weight Calculation
        r.update_mutation_weights();

        r
    }

    /// Allows external processes (like the Genetic Algorithm) to overwrite
    /// the current layout with a new one (e.g. a child from crossover).
    /// This recalculates all internal state (score, weights, etc).
    pub fn inject_layout(&mut self, new_layout: &[u8]) {
        self.layout = new_layout.to_vec();
        self.pos_map = mutation::build_pos_map(&self.layout);

        let (base, left, total) = self.scorer.score_full(&self.pos_map, self.current_limit);
        let imb = self.imbalance_penalty(left);

        self.score = base + imb;
        self.left_load = left;
        self.total_freq = total;

        // Reset mutation weights since layout changed completely
        self.update_mutation_weights();
    }

    // Calculate weights for Cost-Guided Mutation
    pub fn update_mutation_weights(&mut self) {
        let costs = self.scorer.get_element_costs(&self.pos_map);
        let mut sum = 0.0;

        for (i, &c) in costs.iter().enumerate() {
            // Keys locked by user should not be picked for mutation
            if self.locked_indices.contains(&i) {
                self.mutation_weights[i] = 0.0;
            } else {
                // Amplify cost to bias probability significantly
                // Adding 1.0 ensures even "perfect" keys have a non-zero (but small) chance
                self.mutation_weights[i] = (c + 1.0).powf(1.5);
            }
            sum += self.mutation_weights[i];
        }
        self.total_weight = sum;
    }

    // Weighted RNG Selection
    #[inline(always)]
    fn pick_weighted_index(&mut self) -> usize {
        if self.total_weight <= 0.0 {
            return self.rng.usize(0..self.scorer.key_count);
        }

        let target = self.rng.f32() * self.total_weight;
        let mut current = 0.0;

        for (i, &w) in self.mutation_weights.iter().enumerate() {
            current += w;
            if current >= target {
                return i;
            }
        }
        // Fallback for float rounding errors
        self.rng.usize(0..self.scorer.key_count)
    }

    #[inline(always)]
    fn calc_monogram_delta(&self, idx_a: usize, idx_b: usize, char_a: usize, char_b: usize) -> f32 {
        let mut d = 0.0;
        let freq_a = self.scorer.char_freqs[char_a];
        let freq_b = self.scorer.char_freqs[char_b];

        d += (self.local_monogram_costs[idx_b] - self.local_monogram_costs[idx_a]) * freq_a;
        d += (self.local_monogram_costs[idx_a] - self.local_monogram_costs[idx_b]) * freq_b;

        let tier_char_a = self.scorer.char_tier_map[char_a] as usize;
        let tier_char_b = self.scorer.char_tier_map[char_b] as usize;

        if tier_char_a < 3 && tier_char_b < 3 {
            let tier_slot_a = self.scorer.slot_tier_map[idx_a] as usize;
            let tier_slot_b = self.scorer.slot_tier_map[idx_b] as usize;

            if tier_char_a < 3 {
                d -= self.scorer.tier_penalty_matrix[tier_char_a][tier_slot_a] * freq_a;
                d += self.scorer.tier_penalty_matrix[tier_char_a][tier_slot_b] * freq_a;
            }
            if tier_char_b < 3 {
                d -= self.scorer.tier_penalty_matrix[tier_char_b][tier_slot_b] * freq_b;
                d += self.scorer.tier_penalty_matrix[tier_char_b][tier_slot_a] * freq_b;
            }
        }
        d
    }

    #[inline(always)]
    fn calc_bigram_delta(&self, idx_a: usize, idx_b: usize, char_a: usize, char_b: usize) -> f32 {
        let n = self.scorer.key_count;
        let mut d = 0.0;

        let start_a = self.scorer.bigram_starts[char_a];
        let end_a = self.scorer.bigram_starts[char_a + 1];
        let others_a = &self.scorer.bigrams_others[start_a..end_a];
        let freqs_a = &self.scorer.bigrams_freqs[start_a..end_a];
        let self_first_a = &self.scorer.bigrams_self_first[start_a..end_a];

        for i in 0..others_a.len() {
            let other = others_a[i] as usize;
            let p_other = self.pos_map[other] as usize;
            if p_other != 255 {
                let freq = freqs_a[i];
                if self_first_a[i] {
                    d -= self.local_cost_matrix[idx_a * n + p_other] * freq;
                    d += self.local_cost_matrix[idx_b * n + p_other] * freq;
                } else {
                    d -= self.local_cost_matrix[p_other * n + idx_a] * freq;
                    d += self.local_cost_matrix[p_other * n + idx_b] * freq;
                }
            }
        }

        let start_b = self.scorer.bigram_starts[char_b];
        let end_b = self.scorer.bigram_starts[char_b + 1];
        let others_b = &self.scorer.bigrams_others[start_b..end_b];
        let freqs_b = &self.scorer.bigrams_freqs[start_b..end_b];
        let self_first_b = &self.scorer.bigrams_self_first[start_b..end_b];

        for i in 0..others_b.len() {
            let other = others_b[i] as usize;
            let p_other = self.pos_map[other] as usize;
            if p_other != 255 {
                let freq = freqs_b[i];
                if self_first_b[i] {
                    d -= self.local_cost_matrix[idx_b * n + p_other] * freq;
                    d += self.local_cost_matrix[idx_a * n + p_other] * freq;
                } else {
                    d -= self.local_cost_matrix[p_other * n + idx_b] * freq;
                    d += self.local_cost_matrix[p_other * n + idx_a] * freq;
                }
            }
        }

        let freq_ab = self.scorer.freq_matrix[char_a * 256 + char_b];
        if freq_ab > 0.0 {
            let cab = self.local_cost_matrix[idx_a * n + idx_b];
            let cba = self.local_cost_matrix[idx_b * n + idx_a];
            let caa = self.local_cost_matrix[idx_a * n + idx_a];
            let cbb = self.local_cost_matrix[idx_b * n + idx_b];
            d += (cba + cab - cbb - caa) * freq_ab;
        }

        let freq_ba = self.scorer.freq_matrix[char_b * 256 + char_a];
        if freq_ba > 0.0 {
            let cba = self.local_cost_matrix[idx_b * n + idx_a];
            let cab = self.local_cost_matrix[idx_a * n + idx_b];
            let cbb = self.local_cost_matrix[idx_b * n + idx_b];
            let caa = self.local_cost_matrix[idx_a * n + idx_a];
            d += (cab + cba - caa - cbb) * freq_ba;
        }

        let freq_aa = self.scorer.freq_matrix[char_a * 256 + char_a];
        if freq_aa > 0.0 {
            d += (self.local_cost_matrix[idx_b * n + idx_b]
                - self.local_cost_matrix[idx_a * n + idx_a])
                * freq_aa;
        }
        let freq_bb = self.scorer.freq_matrix[char_b * 256 + char_b];
        if freq_bb > 0.0 {
            d += (self.local_cost_matrix[idx_a * n + idx_a]
                - self.local_cost_matrix[idx_b * n + idx_b])
                * freq_bb;
        }
        d
    }

    #[inline(always)]
    fn calc_trigram_delta(
        &self,
        idx_a: usize,
        idx_b: usize,
        char_a: usize,
        char_b: usize,
        limit: usize,
    ) -> f32 {
        let mut d = 0.0;
        let n = self.scorer.key_count;
        let n_sq = n * n;

        let mut process = |c: usize, is_a: bool| {
            let start = self.scorer.trigram_starts[c];
            let end = self.scorer.trigram_starts[c + 1];
            let len = end - start;
            let eff_limit = if len < limit { len } else { limit };

            for t in &self.scorer.trigrams_flat[start..(start + eff_limit)] {
                let o1 = t.other1 as usize;
                let o2 = t.other2 as usize;

                if !is_a && (o1 == char_a || o2 == char_a) {
                    continue;
                }

                let p1_old = self.pos_map[o1] as usize;
                let p2_old = self.pos_map[o2] as usize;

                if p1_old != 255 && p2_old != 255 {
                    let p1_new = if o1 == char_a {
                        idx_b
                    } else if o1 == char_b {
                        idx_a
                    } else {
                        p1_old
                    };
                    let p2_new = if o2 == char_a {
                        idx_b
                    } else if o2 == char_b {
                        idx_a
                    } else {
                        p2_old
                    };

                    let p_c_old = if is_a { idx_a } else { idx_b };
                    let p_c_new = if is_a { idx_b } else { idx_a };

                    let cost_old = match t.role {
                        0 => self.local_trigram_costs[p_c_old * n_sq + p1_old * n + p2_old],
                        1 => self.local_trigram_costs[p1_old * n_sq + p_c_old * n + p2_old],
                        _ => self.local_trigram_costs[p1_old * n_sq + p2_old * n + p_c_old],
                    };

                    let cost_new = match t.role {
                        0 => self.local_trigram_costs[p_c_new * n_sq + p1_new * n + p2_new],
                        1 => self.local_trigram_costs[p1_new * n_sq + p_c_new * n + p2_new],
                        _ => self.local_trigram_costs[p1_new * n_sq + p2_new * n + p_c_new],
                    };

                    d += (cost_new - cost_old) * t.freq;
                }
            }
        };

        process(char_a, true);
        process(char_b, false);

        d
    }

    #[inline(always)]
    pub fn calc_delta(&self, idx_a: usize, idx_b: usize, trigram_limit: usize) -> (f32, f32) {
        let char_a = self.layout[idx_a] as usize;
        let char_b = self.layout[idx_b] as usize;

        let mut delta_score = self.calc_monogram_delta(idx_a, idx_b, char_a, char_b);

        if delta_score > (self.temperature * 10.0) {
            return (f32::INFINITY, 0.0);
        }

        delta_score += self.calc_bigram_delta(idx_a, idx_b, char_a, char_b);
        delta_score += self.calc_trigram_delta(idx_a, idx_b, char_a, char_b, trigram_limit);

        let mut delta_left_load = 0.0;
        let is_left_a = self.scorer.geometry.keys[idx_a].hand == 0;
        let is_left_b = self.scorer.geometry.keys[idx_b].hand == 0;

        if is_left_a && !is_left_b {
            delta_left_load -= self.scorer.char_freqs[char_a];
            delta_left_load += self.scorer.char_freqs[char_b];
        } else if !is_left_a && is_left_b {
            delta_left_load += self.scorer.char_freqs[char_a];
            delta_left_load -= self.scorer.char_freqs[char_b];
        }

        (delta_score, delta_left_load)
    }

    pub fn try_lns_move(&mut self, n_keys: usize) -> bool {
        let key_count = self.scorer.key_count;

        if !(3..=5).contains(&n_keys) || n_keys > key_count {
            return false;
        }

        let mut indices = Vec::with_capacity(n_keys);
        let mut attempts = 0;
        while indices.len() < n_keys && attempts < 50 {
            // LNS Improvement: Pick indices using WEIGHTED PROBABILITY
            // We want to "ruin" the worst parts of the layout
            let idx = self.pick_weighted_index();

            if !indices.contains(&idx) && self.locked_indices.binary_search(&idx).is_err() {
                indices.push(idx);
            }
            attempts += 1;
        }

        if indices.len() != n_keys {
            return false;
        }

        let chars: Vec<u8> = indices.iter().map(|&i| self.layout[i]).collect();
        let mut best_score = self.score;
        let mut best_perm = chars.clone();
        let mut found_better = false;

        for perm in chars.iter().permutations(n_keys) {
            let mut temp_pos = self.pos_map;

            for (k, &char_ref) in perm.iter().enumerate() {
                let char_val = *char_ref;
                let target_idx = indices[k];
                temp_pos[char_val as usize] = target_idx as u8;
            }

            let (raw_score, left, _) = self.scorer.score_full(&temp_pos, self.current_limit);
            let imb = self.imbalance_penalty(left);
            let total = raw_score + imb;

            if total < best_score {
                best_score = total;
                for (k, &char_ref) in perm.iter().enumerate() {
                    best_perm[k] = *char_ref;
                }
                found_better = true;
            }
        }

        if found_better {
            for (k, &char_val) in best_perm.iter().enumerate() {
                let idx = indices[k];
                self.layout[idx] = char_val;
                self.pos_map[char_val as usize] = idx as u8;
            }
            let (_, left, _) = self.scorer.score_full(&self.pos_map, self.current_limit);
            self.left_load = left;
            self.score = best_score;
            return true;
        }

        false
    }

    #[inline(always)]
    pub fn evolve(&mut self, steps: usize) -> (usize, usize) {
        let mut accepted = 0;

        let target_limit = if self.temperature > 10.0 {
            self.limit_fast
        } else {
            self.limit_slow
        };
        if target_limit != self.current_limit {
            self.current_limit = target_limit;
            let (new_base, new_left, _) = self.scorer.score_full(&self.pos_map, target_limit);
            let new_imb = self.imbalance_penalty(new_left);
            self.score = new_base + new_imb;
        }

        // Refresh weights periodically
        let refresh_rate = if self.temperature > 100.0 { 100 } else { 1000 };

        for step in 0..steps {
            if step % refresh_rate == 0 {
                self.update_mutation_weights();
            }

            if self.temperature < 5.0 && self.rng.f32() < 0.002 && self.try_lns_move(4) {
                accepted += 1;
                continue;
            }

            // COST-GUIDED SELECTION
            // Pick 'A' intelligently (problem key)
            let mut idx_a = self.pick_weighted_index();
            // Fallback for safety (though weights for locked should be 0)
            if self.locked_indices.contains(&idx_a) {
                idx_a = self.rng.usize(0..self.scorer.key_count);
            }

            let idx_b = self.rng.usize(0..self.scorer.key_count);

            if idx_a == idx_b {
                continue;
            }

            if self.locked_indices.binary_search(&idx_a).is_ok()
                || self.locked_indices.binary_search(&idx_b).is_ok()
            {
                continue;
            }

            let (delta_base, delta_load) = self.calc_delta(idx_a, idx_b, self.current_limit);
            if delta_base == f32::INFINITY {
                continue;
            }

            let old_imbalance_pen = self.imbalance_penalty(self.left_load);
            let old_base = self.score - old_imbalance_pen;
            let new_base = old_base + delta_base;
            let new_left_load = self.left_load + delta_load;
            let new_imbalance_pen = self.imbalance_penalty(new_left_load);
            let new_total = new_base + new_imbalance_pen;

            let total_delta = new_total - self.score;

            if total_delta < 0.0 || self.rng.f32() < fast_exp(-total_delta / self.temperature) {
                self.layout.swap(idx_a, idx_b);
                let char_a = self.layout[idx_a];
                let char_b = self.layout[idx_b];
                self.pos_map[char_a as usize] = idx_a as u8;
                self.pos_map[char_b as usize] = idx_b as u8;

                let critical = self.scorer.defs.get_critical_bigrams();
                let is_risky = self.scorer.critical_mask[char_a as usize]
                    || self.scorer.critical_mask[char_b as usize];

                if is_risky && fails_sanity(&self.pos_map, &critical, &self.scorer.geometry) {
                    self.layout.swap(idx_a, idx_b);
                    self.pos_map[char_a as usize] = idx_b as u8;
                    self.pos_map[char_b as usize] = idx_a as u8;
                } else {
                    self.score = new_total;
                    self.left_load = new_left_load;
                    accepted += 1;
                }
            }
        }
        (accepted, steps)
    }

    #[inline(always)]
    pub fn imbalance_penalty(&self, left: f32) -> f32 {
        if self.total_freq > 0.0 {
            let ratio = left / self.total_freq;
            let diff = (ratio - 0.5).abs();
            let allowed = self.scorer.weights.allowed_hand_balance_deviation();
            if diff > allowed {
                return diff * self.scorer.weights.penalty_imbalance;
            }
        }
        0.0
    }
}
