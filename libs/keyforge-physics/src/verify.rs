// libs/keyforge-physics/src/verify.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_model::{Corpus, Keyboard, Layout, Rubric};
use keyforge_model::types::{HandIndex, FingerIndex, RowIndex, ColIndex};
use keyforge_model::constants::SCORE_SCALE;

/// A slow, ground-truth implementation of the scoring logic used for validation.
///
/// Unlike the optimized `ScoringEngine`, `DeterministicScorer` does not use 
/// bit-manipulation or kernel caching, making it useful as an oracle for testing.
#[derive(Debug)]
pub struct DeterministicScorer;

impl DeterministicScorer {
    /// Calculates a ground-truth score using the reference implementation.
    #[must_use] 
    #[allow(clippy::cast_precision_loss)]
    pub fn score(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        layout: &Layout,
        cost_matrix: &[(usize, usize, f32)],
    ) -> f32 {
        let raw_score = Self::score_raw(keyboard, corpus, rubric, layout, cost_matrix);
        let total_freq: u64 = corpus.char_freqs.iter().sum();
        let norm_factor = if total_freq > 0 {
            100_000.0 / total_freq as f32
        } else {
            1.0
        };

        raw_score * norm_factor
    }

    /// Calculates the raw, un-normalized score.
    #[allow(clippy::cast_precision_loss)]
    pub fn score_raw(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        layout: &Layout,
        cost_matrix: &[(usize, usize, f32)],
    ) -> f32 {
        let mut total_score: i64 = 0;
        let mut pos_map: Vec<Vec<u16>> = vec![Vec::new(); 65536];
        let limit = layout.keys.len().min(keyboard.keys.len());
        for (i, &code) in layout.keys.iter().enumerate().take(limit) {
            #[allow(clippy::cast_possible_truncation)]
            pos_map[code.0 as usize].push(i as u16);
        }

        let fp_rubric = FixedPointRubric::from(rubric);
        let fp_keys: Vec<FixedPointKey> = keyboard.keys.iter().map(FixedPointKey::from).collect();

        // 1. Monograms
        total_score = total_score.saturating_add(
            Self::score_monograms(keyboard, corpus, rubric, &pos_map)
        );

        // 2. Bigrams
        total_score = total_score.saturating_add(
            Self::score_bigrams(keyboard, corpus, &fp_rubric, &fp_keys, &pos_map, cost_matrix)
        );

        // 3. Trigrams
        total_score = total_score.saturating_add(
            Self::score_trigrams(corpus, &fp_rubric, &fp_keys, &pos_map)
        );

        (total_score as f32) / SCORE_SCALE
    }

    #[allow(clippy::cast_possible_wrap)]
    fn score_monograms(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric, // Using raw rubric for f32 precision in monograms matching original
        pos_map: &[Vec<u16>],
    ) -> i64 {
        let mut total_score: i64 = 0;
        for (char_code, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq == 0 { continue; }
            let candidates = &pos_map[char_code];
            if candidates.is_empty() { continue; }

            let mut min_cost = i64::MAX;
            for &p_idx in candidates {
                let k = &keyboard.keys[p_idx as usize];
                let mut cost = rubric.finger_effort[k.finger.as_usize()];
                if let Some(origin) = keyboard.finger_origins.get(k.hand.as_usize()).and_then(|h| h.get(k.finger.as_usize())) {
                    let dx = (k.x - origin.0).abs();
                    let dy = (k.y - origin.1).abs();
                    cost += (dx * dx * rubric.travel_lat) + (dy * dy * rubric.travel_vert);
                }
                let cost_fixed = to_fixed(cost);
                if cost_fixed < min_cost { min_cost = cost_fixed; }
            }
            total_score = total_score.saturating_add(min_cost.saturating_mul(freq as i64));
        }
        total_score
    }

    #[allow(clippy::cast_possible_wrap)]
    fn score_bigrams(
        keyboard: &Keyboard,
        corpus: &Corpus,
        fp_rubric: &FixedPointRubric,
        fp_keys: &[FixedPointKey],
        pos_map: &[Vec<u16>],
        cost_matrix: &[(usize, usize, f32)],
    ) -> i64 {
        let mut total_score: i64 = 0;
        for &(c1, c2, freq) in &corpus.bigrams {
            let candidates1 = &pos_map[c1 as usize];
            let candidates2 = &pos_map[c2 as usize];
            if candidates1.is_empty() || candidates2.is_empty() { continue; }

            let mut min_cost = i64::MAX;
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let k1 = &fp_keys[p1 as usize];
                    let k2 = &fp_keys[p2 as usize];
                    let mut cost = fp_rubric.calculate_pair_cost(keyboard, k1, k2, p1 as usize, p2 as usize, fp_keys);
                    
                    // Allow cost matrix override
                    for &(o_i, o_j, o_cost) in cost_matrix {
                        if o_i == p1 as usize && o_j == p2 as usize { 
                            cost = to_fixed(o_cost); 
                            break; 
                        }
                    }
                    if cost < min_cost { min_cost = cost; }
                }
            }
            total_score = total_score.saturating_add(min_cost.saturating_mul(i64::from(freq)));
        }
        total_score
    }

    #[allow(clippy::cast_possible_wrap)]
    fn score_trigrams(
        corpus: &Corpus,
        fp_rubric: &FixedPointRubric,
        fp_keys: &[FixedPointKey],
        pos_map: &[Vec<u16>],
    ) -> i64 {
        let mut total_score: i64 = 0;
        for &(c1, c2, c3, freq) in &corpus.trigrams {
            let candidates1 = &pos_map[c1 as usize];
            let candidates2 = &pos_map[c2 as usize];
            let candidates3 = &pos_map[c3 as usize];
            if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() { continue; }

            let mut min_cost = i64::MAX;
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        let k1 = &fp_keys[p1 as usize];
                        let k2 = &fp_keys[p2 as usize];
                        let k3 = &fp_keys[p3 as usize];
                        let cost = fp_rubric.calculate_flow_cost(k1, k2, k3);
                        if cost < min_cost { min_cost = cost; }
                    }
                }
            }
            if min_cost != i64::MAX && min_cost != 0 {
                total_score = total_score.saturating_add(min_cost.saturating_mul(i64::from(freq)));
            }
        }
        total_score
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn to_fixed(val: f32) -> i64 {
    if val.is_nan() { return 0; }
    if val.is_infinite() { return if val.is_sign_positive() { i64::MAX } else { i64::MIN }; }
    let scaled = f64::from(val) * f64::from(SCORE_SCALE);
    // Boundary check using f64 (more precise than f32 at 10^18)
    if scaled >= i64::MAX as f64 { 
        i64::MAX 
    } else if scaled <= i64::MIN as f64 { 
        i64::MIN 
    } else { 
        scaled as i64 
    }
}

#[derive(Debug, Clone)]
struct FixedPointKey {
    x: i64,
    y: i64,
    hand: HandIndex,
    finger: FingerIndex,
    row: RowIndex,
    col: ColIndex,
}

impl From<&keyforge_model::geometry::KeyNode> for FixedPointKey {
    fn from(k: &keyforge_model::geometry::KeyNode) -> Self {
        Self {
            x: to_fixed(k.x),
            y: to_fixed(k.y),
            hand: k.hand,
            finger: k.finger,
            row: k.row,
            col: k.col,
        }
    }
}

#[derive(Debug)]
struct FixedPointRubric {
    travel_lat: i64,
    travel_vert: i64,
    sfb_base: i64,
    sfb_lateral: i64,
    sfb_lateral_weak: i64,
    sfb_diagonal: i64,
    sfb_long: i64,
    threshold_sfb_long_row_diff: i8,
    redirect: i64,
    roll_bonus: i64,
    penalty_scissor: i64,
    threshold_scissor_row_diff: i8,
}

impl From<&Rubric> for FixedPointRubric {
    fn from(rubric: &Rubric) -> Self {
        Self {
            travel_lat: to_fixed(rubric.travel_lat),
            travel_vert: to_fixed(rubric.travel_vert),
            sfb_base: to_fixed(rubric.sfb_base),
            sfb_lateral: to_fixed(rubric.sfb_lateral),
            sfb_lateral_weak: to_fixed(rubric.sfb_lateral_weak),
            sfb_diagonal: to_fixed(rubric.sfb_diagonal),
            sfb_long: to_fixed(rubric.sfb_long),
            threshold_sfb_long_row_diff: rubric.threshold_sfb_long_row_diff,
            redirect: to_fixed(rubric.redirect),
            roll_bonus: to_fixed(rubric.roll_bonus),
            penalty_scissor: to_fixed(rubric.penalty_scissor),
            threshold_scissor_row_diff: rubric.threshold_scissor_row_diff,
        }
    }
}

impl FixedPointRubric {
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_pair_cost(
        &self, 
        kb: &Keyboard, 
        k1: &FixedPointKey, 
        k2: &FixedPointKey, 
        idx1: usize, 
        idx2: usize,
        _fp_keys: &[FixedPointKey] // Needed for lookups if necessary, though k1/k2 usually enough
    ) -> i64 {
        if idx1 == idx2 { return 0; }
        
        // Travel Distance (Bigram common)
        let dx = (k1.x - k2.x).abs();
        let dy = (k1.y - k2.y).abs();
        let scale_val = SCORE_SCALE as i128;
        let scale_sq = scale_val * scale_val;
        let term_x = i128::from(dx).saturating_mul(i128::from(dx)).saturating_mul(i128::from(self.travel_lat)) / scale_sq;
        let term_y = i128::from(dy).saturating_mul(i128::from(dy)).saturating_mul(i128::from(self.travel_vert)) / scale_sq;
        let dist_cost = (term_x.saturating_add(term_y)) as i64;
        
        if k1.hand != k2.hand { return 0; }
        
        let mut cost = dist_cost; 

        if k1.finger == k2.finger {
            // SFB Correction: Subtract monogram reach of K2
            let mut reach_k2 = 0i64;
            if let Some(origin) = kb.finger_origins.get(k2.hand.as_usize()).and_then(|h| h.get(k2.finger.as_usize())) {
                // Warning: We are mixing f32 origin with fixed keys, essentially recalculating.
                // Replicating original logic: use raw keyboard keys [idx2] for consistency with mono logic if needed.
                // But k2.x is to_fixed(kb.keys[idx2].x).
                // Original: rdx = to_fixed((kb.keys[idx2].x - origin.0).abs())
                // Let's rely on kb keys to be exact.
                let rdx = to_fixed((kb.keys[idx2].x - origin.0).abs());
                let rdy = to_fixed((kb.keys[idx2].y - origin.1).abs());
                let r_term_x = i128::from(rdx).saturating_mul(i128::from(rdx)).saturating_mul(i128::from(self.travel_lat)) / scale_sq;
                let r_term_y = i128::from(rdy).saturating_mul(i128::from(rdy)).saturating_mul(i128::from(self.travel_vert)) / scale_sq;
                reach_k2 = (r_term_x.saturating_add(r_term_y)) as i64;
            }
            
            cost = dist_cost.saturating_sub(reach_k2);

            let row_diff = (k1.row.0 - k2.row.0).abs();
            let col_diff = (k1.col.0 - k2.col.0).abs();

            if col_diff == 1 {
                if k1.finger.is_weak() { cost = cost.saturating_add(self.sfb_lateral_weak); }
                else { cost = cost.saturating_add(self.sfb_lateral); }
            } else if col_diff > 1 {
                cost = cost.saturating_add(self.sfb_diagonal);
            } else if row_diff >= self.threshold_sfb_long_row_diff {
                cost = cost.saturating_add(self.sfb_long);
            } else {
                cost = cost.saturating_add(self.sfb_base);
            }
            return cost;
        }

        let finger_diff = k1.finger.distance(k2.finger);
        let row_diff = (k1.row.0 - k2.row.0).abs();
        if finger_diff == 1 && k1.finger != FingerIndex::THUMB && k2.finger != FingerIndex::THUMB {
            if row_diff >= self.threshold_scissor_row_diff {
                cost = cost.saturating_add(self.penalty_scissor);
            } else if row_diff == 0 {
                let col_dist = (k1.col.0 - k2.col.0).abs();
                if col_dist > 1 { cost = cost.saturating_add(self.sfb_lateral); }
            }
        }
        cost
    }

    fn calculate_flow_cost(&self, k1: &FixedPointKey, k2: &FixedPointKey, k3: &FixedPointKey) -> i64 {
        if k1.hand != k2.hand || k2.hand != k3.hand { return 0; }
        if k1.finger == k3.finger && k1.finger != k2.finger { return self.redirect; }
        let dir1 = k2.finger.diff(k1.finger);
        let dir2 = k3.finger.diff(k2.finger);
        if dir1 == 0 || dir2 == 0 { return 0; }
        if dir1.signum() != dir2.signum() { return self.redirect; }
        if dir1 < 0 { return self.roll_bonus.saturating_neg(); }
        0
    }
}
