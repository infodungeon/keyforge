pub mod mutation;

use self::mutation::*;
use crate::scorer::Scorer;
use std::sync::Arc;

#[inline(always)]
fn fast_exp(x: f32) -> f32 {
    let x = 1.0 + x / 256.0;
    let x = x * x;
    let x = x * x;
    let x = x * x;
    let x = x * x;
    let x = x * x;
    let x = x * x;
    let x = x * x;
    x * x
}

#[repr(align(64))]
pub struct Replica {
    pub scorer: Arc<Scorer>,
    pub local_cost_matrix: [[f32; 30]; 30],
    pub local_trigram_costs: Vec<f32>,
    pub local_monogram_costs: [f32; 30],

    pub layout: [u8; 30],
    pub pos_map: [u8; 256],
    pub score: f32,
    pub left_load: f32,
    pub total_freq: f32,
    pub temperature: f32,
    pub debug: bool,

    // Optimization limits now stored locally since Scorer doesn't hold SearchParams
    pub current_limit: usize,
    pub limit_fast: usize,
    pub limit_slow: usize,

    pub rng: fastrand::Rng,
}

impl Replica {
    pub fn new(
        scorer: Arc<Scorer>,
        temperature: f32,
        seed: Option<u64>,
        debug: bool,
        limit_fast: usize,
        limit_slow: usize,
    ) -> Self {
        let mut rng = if let Some(s) = seed {
            fastrand::Rng::with_seed(s)
        } else {
            fastrand::Rng::new()
        };

        let mut layout;
        let mut pos_map;
        loop {
            // DYNAMIC GEOMETRY: Pass geometry to generator
            layout = mutation::generate_tiered_layout(&mut rng, &scorer.defs, &scorer.geometry);
            pos_map = mutation::build_pos_map(&layout);

            let critical = scorer.defs.get_critical_bigrams();
            // DYNAMIC GEOMETRY: Pass geometry to sanity check
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

        let local_cost_matrix = scorer.full_cost_matrix;
        let local_trigram_costs = scorer.trigram_cost_table.clone();
        let local_monogram_costs = scorer.slot_monogram_costs;

        let mut r = Replica {
            scorer,
            local_cost_matrix,
            local_trigram_costs,
            local_monogram_costs,
            layout,
            pos_map,
            score: base, // Temporarily Base
            left_load: left,
            total_freq: total,
            temperature,
            debug,
            current_limit: start_limit,
            limit_fast,
            limit_slow,
            rng,
        };

        // Calculate initial imbalance and add to score so it represents Total Score
        let imb = r.imbalance_penalty(left);
        r.score += imb;

        r
    }

    pub fn check_integrity(&self) -> (f32, f32) {
        let (base, left, _) = self.scorer.score_full(&self.pos_map, self.current_limit);

        let imb = self.imbalance_penalty(left);
        let real_total = base + imb;

        let diff = (self.score - real_total).abs();
        (diff, real_total)
    }

    #[inline(always)]
    pub fn calc_delta(&self, idx_a: usize, idx_b: usize, trigram_limit: usize) -> (f32, f32) {
        let char_a = self.layout[idx_a] as usize;
        let char_b = self.layout[idx_b] as usize;
        let mut delta_score = 0.0;

        // 1. Monogram Delta (Reach + Effort + Tier)
        let freq_a = self.scorer.char_freqs[char_a];
        let freq_b = self.scorer.char_freqs[char_b];

        // A goes from idx_a to idx_b.
        delta_score +=
            (self.local_monogram_costs[idx_b] - self.local_monogram_costs[idx_a]) * freq_a;
        // B goes from idx_b to idx_a.
        delta_score +=
            (self.local_monogram_costs[idx_a] - self.local_monogram_costs[idx_b]) * freq_b;

        // Tier Delta
        let tier_char_a = self.scorer.char_tier_map[char_a] as usize;
        let tier_char_b = self.scorer.char_tier_map[char_b] as usize;

        if tier_char_a < 3 && tier_char_b < 3 {
            let tier_slot_a = self.scorer.slot_tier_map[idx_a] as usize;
            let tier_slot_b = self.scorer.slot_tier_map[idx_b] as usize;

            if tier_char_a < 3 {
                delta_score -= self.scorer.tier_penalty_matrix[tier_char_a][tier_slot_a] * freq_a;
                delta_score += self.scorer.tier_penalty_matrix[tier_char_a][tier_slot_b] * freq_a;
            }
            if tier_char_b < 3 {
                delta_score -= self.scorer.tier_penalty_matrix[tier_char_b][tier_slot_b] * freq_b;
                delta_score += self.scorer.tier_penalty_matrix[tier_char_b][tier_slot_a] * freq_b;
            }
        }

        if delta_score > (self.temperature * 10.0) {
            return (f32::INFINITY, 0.0);
        }

        // 2. Bigrams
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
                    delta_score -= self.local_cost_matrix[idx_a][p_other] * freq;
                    delta_score += self.local_cost_matrix[idx_b][p_other] * freq;
                } else {
                    delta_score -= self.local_cost_matrix[p_other][idx_a] * freq;
                    delta_score += self.local_cost_matrix[p_other][idx_b] * freq;
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
                    delta_score -= self.local_cost_matrix[idx_b][p_other] * freq;
                    delta_score += self.local_cost_matrix[idx_a][p_other] * freq;
                } else {
                    delta_score -= self.local_cost_matrix[p_other][idx_b] * freq;
                    delta_score += self.local_cost_matrix[p_other][idx_a] * freq;
                }
            }
        }

        let freq_ab = self.scorer.freq_matrix[char_a][char_b];
        if freq_ab > 0.0 {
            let cab = self.local_cost_matrix[idx_a][idx_b];
            let cba = self.local_cost_matrix[idx_b][idx_a];
            let caa = self.local_cost_matrix[idx_a][idx_a];
            let cbb = self.local_cost_matrix[idx_b][idx_b];
            delta_score += (cba + cab - cbb - caa) * freq_ab;
        }

        let freq_ba = self.scorer.freq_matrix[char_b][char_a];
        if freq_ba > 0.0 {
            let cba = self.local_cost_matrix[idx_b][idx_a];
            let cab = self.local_cost_matrix[idx_a][idx_b];
            let cbb = self.local_cost_matrix[idx_b][idx_b];
            let caa = self.local_cost_matrix[idx_a][idx_a];
            delta_score += (cab + cba - caa - cbb) * freq_ba;
        }

        let freq_aa = self.scorer.freq_matrix[char_a][char_a];
        if freq_aa > 0.0 {
            delta_score += (self.local_cost_matrix[idx_b][idx_b]
                - self.local_cost_matrix[idx_a][idx_a])
                * freq_aa;
        }

        let freq_bb = self.scorer.freq_matrix[char_b][char_b];
        if freq_bb > 0.0 {
            delta_score += (self.local_cost_matrix[idx_a][idx_a]
                - self.local_cost_matrix[idx_b][idx_b])
                * freq_bb;
        }

        // 3. Trigrams
        let start = self.scorer.trigram_starts[char_a];
        let end = self.scorer.trigram_starts[char_a + 1];
        let len = end - start;
        let limit = if len < trigram_limit {
            len
        } else {
            trigram_limit
        };

        for t in &self.scorer.trigrams_flat[start..(start + limit)] {
            let o1 = t.other1 as usize;
            let o2 = t.other2 as usize;
            let p1_old = self.pos_map[o1] as usize;
            let p2_old = self.pos_map[o2] as usize;

            if p1_old != 255 && p2_old != 255 {
                let p1_new = if o1 == char_b {
                    idx_a
                } else if o1 == char_a {
                    idx_b
                } else {
                    p1_old
                };

                let p2_new = if o2 == char_b {
                    idx_a
                } else if o2 == char_a {
                    idx_b
                } else {
                    p2_old
                };

                let cost_old = match t.role {
                    0 => self.local_trigram_costs[idx_a * 900 + p1_old * 30 + p2_old],
                    1 => self.local_trigram_costs[p1_old * 900 + idx_a * 30 + p2_old],
                    _ => self.local_trigram_costs[p1_old * 900 + p2_old * 30 + idx_a],
                };
                let cost_new = match t.role {
                    0 => self.local_trigram_costs[idx_b * 900 + p1_new * 30 + p2_new],
                    1 => self.local_trigram_costs[p1_new * 900 + idx_b * 30 + p2_new],
                    _ => self.local_trigram_costs[p1_new * 900 + p2_new * 30 + idx_b],
                };

                delta_score += (cost_new - cost_old) * t.freq;
            }
        }

        let start = self.scorer.trigram_starts[char_b];
        let end = self.scorer.trigram_starts[char_b + 1];
        let len = end - start;
        let limit = if len < trigram_limit {
            len
        } else {
            trigram_limit
        };

        for t in &self.scorer.trigrams_flat[start..(start + limit)] {
            let o1 = t.other1 as usize;
            let o2 = t.other2 as usize;
            if o1 == char_a || o2 == char_a {
                continue;
            }

            let p1_old = self.pos_map[o1] as usize;
            let p2_old = self.pos_map[o2] as usize;

            if p1_old != 255 && p2_old != 255 {
                let p1_new = if o1 == char_b { idx_a } else { p1_old };
                let p2_new = if o2 == char_b { idx_a } else { p2_old };

                let cost_old = match t.role {
                    0 => self.local_trigram_costs[idx_b * 900 + p1_old * 30 + p2_old],
                    1 => self.local_trigram_costs[p1_old * 900 + idx_b * 30 + p2_old],
                    _ => self.local_trigram_costs[p1_old * 900 + p2_old * 30 + idx_b],
                };
                let cost_new = match t.role {
                    0 => self.local_trigram_costs[idx_a * 900 + p1_new * 30 + p2_new],
                    1 => self.local_trigram_costs[p1_new * 900 + idx_a * 30 + p2_new],
                    _ => self.local_trigram_costs[p1_new * 900 + p2_new * 30 + idx_a],
                };

                delta_score += (cost_new - cost_old) * t.freq;
            }
        }

        // Load Balance Delta
        let mut delta_left_load = 0.0;
        // DYNAMIC: Check hand assignment from geometry
        let is_left_a = self.scorer.geometry.keys[idx_a].hand == 0;
        let is_left_b = self.scorer.geometry.keys[idx_b].hand == 0;

        if is_left_a && !is_left_b {
            delta_left_load -= freq_a;
            delta_left_load += freq_b;
        } else if !is_left_a && is_left_b {
            delta_left_load += freq_a;
            delta_left_load -= freq_b;
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

            // Ensure we reset to Base + Imbalance when limits change
            let new_imb = self.imbalance_penalty(new_left);
            self.score = new_base + new_imb;
        }

        for _ in 0..steps {
            let idx_a = self.rng.usize(0..30);
            let idx_b = self.rng.usize(0..30);

            if idx_a == idx_b {
                continue;
            }

            let (delta_base, delta_load) = self.calc_delta(idx_a, idx_b, self.current_limit);

            if delta_base == f32::INFINITY {
                continue;
            }

            // self.score is TOTAL.
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

                // DYNAMIC: Pass geometry to sanity check
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
    fn imbalance_penalty(&self, left: f32) -> f32 {
        if self.total_freq > 0.0 {
            let ratio = left / self.total_freq;
            let dist = (ratio - 0.5).abs();
            if dist > (self.scorer.weights.max_hand_imbalance - 0.5) {
                return dist * self.scorer.weights.penalty_imbalance;
            }
        }
        0.0
    }
}
