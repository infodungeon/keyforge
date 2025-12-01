pub mod mutation;
pub mod runner; // NEW

use self::mutation::*;
pub use self::runner::{OptimizationOptions, Optimizer, ProgressCallback};

use crate::scorer::Scorer;
use std::sync::Arc;

// ... [Replica struct definition and impl remain the same] ...
// (Ensure fast_exp and Replica code from previous stages is preserved here)

#[inline(always)]
fn fast_exp(x: f32) -> f32 {
    let x = 1.0 + x / 256.0;
    let x = x * x * x * x * x * x * x * x;
    x * x
}

#[repr(align(64))]
pub struct Replica {
    // ... [fields unchanged]
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
}

impl Replica {
    // ... [new() and evolve() methods unchanged from Stage 4] ...
    // (Paste the full Replica impl block here if modifying the file)
    pub fn new(
        scorer: Arc<Scorer>,
        temperature: f32,
        seed: Option<u64>,
        debug: bool,
        limit_fast: usize,
        limit_slow: usize,
        pinned_keys_str: &str,
    ) -> Self {
        // ... (implementation matches Stage 4) ...
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
        };

        let imb = r.imbalance_penalty(left);
        r.score += imb;

        r
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

        let key_count = self.scorer.key_count;

        for _ in 0..steps {
            let idx_a = self.rng.usize(0..key_count);
            let idx_b = self.rng.usize(0..key_count);

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
