use super::Replica;
use crate::consts::KEY_NOT_FOUND_U8;

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
            unsafe { *self.scorer.char_freqs.get_unchecked(char_a) }
        } else {
            0.0
        };
        let freq_b = if char_b < 256 {
            unsafe { *self.scorer.char_freqs.get_unchecked(char_b) }
        } else {
            0.0
        };

        unsafe {
            d += (*self.scorer.slot_monogram_costs.get_unchecked(idx_b)
                - *self.scorer.slot_monogram_costs.get_unchecked(idx_a))
                * freq_a;
            d += (*self.scorer.slot_monogram_costs.get_unchecked(idx_a)
                - *self.scorer.slot_monogram_costs.get_unchecked(idx_b))
                * freq_b;
        }

        if char_a < 256 && char_b < 256 {
            unsafe {
                let tier_char_a = *self.scorer.char_tier_map.get_unchecked(char_a) as usize;
                let tier_char_b = *self.scorer.char_tier_map.get_unchecked(char_b) as usize;

                if tier_char_a < 3 && tier_char_b < 3 {
                    let tier_slot_a = *self.scorer.slot_tier_map.get_unchecked(idx_a) as usize;
                    let tier_slot_b = *self.scorer.slot_tier_map.get_unchecked(idx_b) as usize;

                    if tier_char_a < 3 {
                        d -= self
                            .scorer
                            .tier_penalty_matrix
                            .get_unchecked(tier_char_a)
                            .get_unchecked(tier_slot_a)
                            * freq_a;
                        d += self
                            .scorer
                            .tier_penalty_matrix
                            .get_unchecked(tier_char_a)
                            .get_unchecked(tier_slot_b)
                            * freq_a;
                    }
                    if tier_char_b < 3 {
                        d -= self
                            .scorer
                            .tier_penalty_matrix
                            .get_unchecked(tier_char_b)
                            .get_unchecked(tier_slot_b)
                            * freq_b;
                        d += self
                            .scorer
                            .tier_penalty_matrix
                            .get_unchecked(tier_char_b)
                            .get_unchecked(tier_slot_a)
                            * freq_b;
                    }
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
        let cost_ptr = self.scorer.full_cost_matrix.as_ptr();

        let mut process_neighbors = |c_main: usize, idx_old: usize, idx_new: usize| {
            if c_main >= 256 {
                return;
            }
            unsafe {
                let start = *self.scorer.bigram_starts.get_unchecked(c_main);
                let end = *self.scorer.bigram_starts.get_unchecked(c_main + 1);

                for i in start..end {
                    let other = *self.scorer.bigrams_others.get_unchecked(i) as usize;
                    let p_other = *self.compact_map.get_unchecked(other) as usize;

                    if p_other != KEY_NOT_FOUND_U8 as usize {
                        let freq = *self.scorer.bigrams_freqs.get_unchecked(i);
                        if *self.scorer.bigrams_self_first.get_unchecked(i) {
                            let c_old = *cost_ptr.add(idx_old * n + p_other);
                            let c_new = *cost_ptr.add(idx_new * n + p_other);
                            d += (c_new - c_old) * freq;
                        } else {
                            let c_old = *cost_ptr.add(p_other * n + idx_old);
                            let c_new = *cost_ptr.add(p_other * n + idx_new);
                            d += (c_new - c_old) * freq;
                        }
                    }
                }
            }
        };

        process_neighbors(char_a, idx_a, idx_b);
        process_neighbors(char_b, idx_b, idx_a);

        if char_a < 256 && char_b < 256 {
            unsafe {
                let freq_ab = *self.scorer.freq_matrix.get_unchecked(char_a * 256 + char_b);
                if freq_ab > 0.0 {
                    let cab = *cost_ptr.add(idx_a * n + idx_b);
                    let cba = *cost_ptr.add(idx_b * n + idx_a);
                    let caa = *cost_ptr.add(idx_a * n + idx_a);
                    let cbb = *cost_ptr.add(idx_b * n + idx_b);
                    d += (cba + cab - cbb - caa) * freq_ab;
                }

                let freq_ba = *self.scorer.freq_matrix.get_unchecked(char_b * 256 + char_a);
                if freq_ba > 0.0 {
                    let cba = *cost_ptr.add(idx_b * n + idx_a);
                    let cab = *cost_ptr.add(idx_a * n + idx_b);
                    let cbb = *cost_ptr.add(idx_b * n + idx_b);
                    let caa = *cost_ptr.add(idx_a * n + idx_a);
                    d += (cab + cba - caa - cbb) * freq_ba;
                }

                let freq_aa = *self.scorer.freq_matrix.get_unchecked(char_a * 256 + char_a);
                if freq_aa > 0.0 {
                    d += (*cost_ptr.add(idx_b * n + idx_b) - *cost_ptr.add(idx_a * n + idx_a))
                        * freq_aa;
                }
                let freq_bb = *self.scorer.freq_matrix.get_unchecked(char_b * 256 + char_b);
                if freq_bb > 0.0 {
                    d += (*cost_ptr.add(idx_a * n + idx_a) - *cost_ptr.add(idx_b * n + idx_b))
                        * freq_bb;
                }
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
        let tri_ptr = self.scorer.trigram_cost_table.as_ptr();

        let mut process = |c: usize, is_a: bool| {
            if c >= 256 {
                return;
            }
            unsafe {
                let start = *self.scorer.trigram_starts.get_unchecked(c);
                let end = *self.scorer.trigram_starts.get_unchecked(c + 1);
                let len = end - start;
                let eff_limit = if len < limit { len } else { limit };

                // Correctly iterate over trigrams_flat instead of legacy fields
                let trigrams = self
                    .scorer
                    .trigrams_flat
                    .get_unchecked(start..(start + eff_limit));

                for t in trigrams {
                    let o1 = t.other1 as usize;
                    let o2 = t.other2 as usize;

                    if !is_a && (o1 == char_a || o2 == char_a) {
                        continue;
                    }

                    let p1_old = *self.compact_map.get_unchecked(o1) as usize;
                    let p2_old = *self.compact_map.get_unchecked(o2) as usize;

                    if p1_old != KEY_NOT_FOUND_U8 as usize && p2_old != KEY_NOT_FOUND_U8 as usize {
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

                        let (cost_old, cost_new) = match t.role {
                            0 => (
                                *tri_ptr.add(p_c_old * n_sq + p1_old * n + p2_old),
                                *tri_ptr.add(p_c_new * n_sq + p1_new * n + p2_new),
                            ),
                            1 => (
                                *tri_ptr.add(p1_old * n_sq + p_c_old * n + p2_old),
                                *tri_ptr.add(p1_new * n_sq + p_c_new * n + p2_new),
                            ),
                            _ => (
                                *tri_ptr.add(p1_old * n_sq + p2_old * n + p_c_old),
                                *tri_ptr.add(p1_new * n_sq + p2_new * n + p_c_new),
                            ),
                        };

                        d += (cost_new - cost_old) * t.freq;
                    }
                }
            }
        };

        process(char_a, true);
        process(char_b, false);
        d
    }

    #[inline(always)]
    pub fn calc_delta(&self, idx_a: usize, idx_b: usize, trigram_limit: usize) -> (f32, f32) {
        unsafe {
            let char_a = *self.layout.get_unchecked(idx_a) as usize;
            let char_b = *self.layout.get_unchecked(idx_b) as usize;

            let mut delta_score = self.calc_monogram_delta(idx_a, idx_b, char_a, char_b);

            if delta_score > (self.temperature * 50.0) {
                return (f32::INFINITY, 0.0);
            }

            delta_score += self.calc_bigram_delta(idx_a, idx_b, char_a, char_b);

            if delta_score > (self.temperature * 50.0) {
                return (f32::INFINITY, 0.0);
            }

            delta_score += self.calc_trigram_delta(idx_a, idx_b, char_a, char_b, trigram_limit);

            // Hand Balance Delta
            let mut delta_left_load = 0.0;
            // Retrieve hand info from Geometry via index, not legacy arrays
            let is_left_a = self.scorer.geometry.keys.get_unchecked(idx_a).hand == 0;
            let is_left_b = self.scorer.geometry.keys.get_unchecked(idx_b).hand == 0;

            if is_left_a != is_left_b {
                if char_a < 256 {
                    let f = *self.scorer.char_freqs.get_unchecked(char_a);
                    if is_left_a {
                        delta_left_load -= f;
                    } else {
                        delta_left_load += f;
                    }
                }
                if char_b < 256 {
                    let f = *self.scorer.char_freqs.get_unchecked(char_b);
                    if is_left_b {
                        delta_left_load -= f;
                    } else {
                        delta_left_load += f;
                    }
                }
            }

            (delta_score, delta_left_load)
        }
    }
}
