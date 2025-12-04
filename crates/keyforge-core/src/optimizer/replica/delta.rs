use super::Replica;

impl Replica {
    #[inline(always)]
    pub(crate) fn calc_monogram_delta(
        &self,
        idx_a: usize,
        idx_b: usize,
        char_a: usize,
        char_b: usize,
    ) -> f32 {
        let mut d = 0.0;

        if char_a >= 256 && char_b >= 256 {
            return 0.0;
        }

        let freq_a = if char_a < 256 {
            self.scorer.char_freqs[char_a]
        } else {
            0.0
        };
        let freq_b = if char_b < 256 {
            self.scorer.char_freqs[char_b]
        } else {
            0.0
        };

        d += (self.local_monogram_costs[idx_b] - self.local_monogram_costs[idx_a]) * freq_a;
        d += (self.local_monogram_costs[idx_a] - self.local_monogram_costs[idx_b]) * freq_b;

        // Tier Penalty
        if char_a < 256 && char_b < 256 {
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
        }
        d
    }

    #[inline(always)]
    pub(crate) fn calc_bigram_delta(
        &self,
        idx_a: usize,
        idx_b: usize,
        char_a: usize,
        char_b: usize,
    ) -> f32 {
        if char_a >= 256 && char_b >= 256 {
            return 0.0;
        }

        let n = self.scorer.key_count;
        let mut d = 0.0;

        let mut process_neighbors = |c_main: usize, idx_old: usize, idx_new: usize| {
            if c_main >= 256 {
                return;
            }
            let start = self.scorer.bigram_starts[c_main];
            let end = self.scorer.bigram_starts[c_main + 1];

            for i in start..end {
                let other = self.scorer.bigrams_others[i] as usize;
                let p_other = self.pos_map[other] as usize;

                if p_other != 255 {
                    let freq = self.scorer.bigrams_freqs[i];
                    if self.scorer.bigrams_self_first[i] {
                        d -= self.local_cost_matrix[idx_old * n + p_other] * freq;
                        d += self.local_cost_matrix[idx_new * n + p_other] * freq;
                    } else {
                        d -= self.local_cost_matrix[p_other * n + idx_old] * freq;
                        d += self.local_cost_matrix[p_other * n + idx_new] * freq;
                    }
                }
            }
        };

        process_neighbors(char_a, idx_a, idx_b);
        process_neighbors(char_b, idx_b, idx_a);

        if char_a < 256 && char_b < 256 {
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
        }
        d
    }

    #[inline(always)]
    pub(crate) fn calc_trigram_delta(
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
            if c >= 256 {
                return;
            }
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
            if char_a < 256 {
                delta_left_load -= self.scorer.char_freqs[char_a];
            }
            if char_b < 256 {
                delta_left_load += self.scorer.char_freqs[char_b];
            }
        } else if !is_left_a && is_left_b {
            if char_a < 256 {
                delta_left_load += self.scorer.char_freqs[char_a];
            }
            if char_b < 256 {
                delta_left_load -= self.scorer.char_freqs[char_b];
            }
        }

        (delta_score, delta_left_load)
    }
}
