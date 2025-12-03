use super::Replica;
use crate::consts::ANNEAL_TEMP_SCALE;
use crate::core_types::KeyCode;
use crate::optimizer::mutation;
use itertools::Itertools;

#[inline(always)]
fn fast_exp(x: f32) -> f32 {
    // Optimized approximation: (1 + x/256)^256
    let x = 1.0 + x / ANNEAL_TEMP_SCALE;
    let x = x * x * x * x * x * x * x * x;
    x * x
}

impl Replica {
    pub(crate) fn pick_weighted_index(&mut self) -> usize {
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
        self.rng.usize(0..self.scorer.key_count)
    }

    pub fn try_lns_move(&mut self, n_keys: usize) -> bool {
        let key_count = self.scorer.key_count;
        if !(3..=5).contains(&n_keys) || n_keys > key_count {
            return false;
        }

        let mut indices = Vec::with_capacity(n_keys);
        let mut attempts = 0;
        while indices.len() < n_keys && attempts < 50 {
            let idx = self.pick_weighted_index();
            if !indices.contains(&idx) && self.locked_indices.binary_search(&idx).is_err() {
                indices.push(idx);
            }
            attempts += 1;
        }

        if indices.len() != n_keys {
            return false;
        }

        let chars: Vec<KeyCode> = indices.iter().map(|&i| self.layout[i]).collect();
        let mut best_score = self.score;
        let mut best_perm = chars.clone();
        let mut found_better = false;

        for perm in chars.iter().permutations(n_keys) {
            let mut temp_pos = self.pos_map.clone();
            for (k, &char_ref) in perm.iter().enumerate() {
                let char_val = *char_ref;
                let target_idx = indices[k];
                temp_pos[char_val as usize] = target_idx as u8;
            }

            let (raw_score, left, _) = self.scorer.score_full(&temp_pos, self.current_limit);
            let total = raw_score + self.imbalance_penalty(left);

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
            self.score = new_base + self.imbalance_penalty(new_left);
        }

        let refresh_rate = if self.temperature > 100.0 { 100 } else { 1000 };

        for step in 0..steps {
            if step % refresh_rate == 0 {
                self.update_mutation_weights();
            }

            if self.temperature < 5.0 && self.rng.f32() < 0.002 && self.try_lns_move(4) {
                accepted += 1;
                continue;
            }

            let mut idx_a = self.pick_weighted_index();
            if self.locked_indices.contains(&idx_a) {
                idx_a = self.rng.usize(0..self.scorer.key_count);
            }
            let idx_b = self.rng.usize(0..self.scorer.key_count);

            if idx_a == idx_b
                || self.locked_indices.contains(&idx_a)
                || self.locked_indices.contains(&idx_b)
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
            let new_total = new_base + self.imbalance_penalty(new_left_load);

            let total_delta = new_total - self.score;

            if total_delta < 0.0 || self.rng.f32() < fast_exp(-total_delta / self.temperature) {
                self.layout.swap(idx_a, idx_b);
                let char_a = self.layout[idx_a];
                let char_b = self.layout[idx_b];
                self.pos_map[char_a as usize] = idx_a as u8;
                self.pos_map[char_b as usize] = idx_b as u8;

                let critical = self.scorer.defs.get_critical_bigrams();
                let mut is_risky = false;
                if char_a < 256 {
                    is_risky |= self.scorer.critical_mask[char_a as usize];
                }
                if char_b < 256 {
                    is_risky |= self.scorer.critical_mask[char_b as usize];
                }

                if is_risky
                    && mutation::fails_sanity(&self.pos_map, &critical, &self.scorer.geometry)
                {
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
}
