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

use keyforge_model::constants::SCORE_SCALE;
use keyforge_model::cost_model::{FingerDefinition, HandDefinition};
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};
use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};

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
        cost_model: &CostModel,
    ) -> f32 {
        let raw_score_scaled = Self::score_raw_scaled(keyboard, corpus, rubric, layout, cost_model);
        let raw_score = raw_score_scaled as f32 / SCORE_SCALE;

        let total_freq: u64 = corpus.char_freqs.iter().sum();
        let norm_factor = if total_freq > 0 {
            100_000.0 / total_freq as f32
        } else {
            1.0
        };

        raw_score * norm_factor
    }

    /// Calculates the raw, scaled i64 score.
    pub fn score_raw_scaled(
        keyboard: &Keyboard,
        corpus: &Corpus,
        rubric: &Rubric,
        layout: &Layout,
        cost_model: &CostModel,
    ) -> i64 {
        let mut total_score: i64 = 0;
        let mut pos_map: Vec<Vec<u16>> = vec![Vec::new(); 65536];
        let limit = layout.keys.len().min(keyboard.keys.len());
        for (i, &code) in layout.keys.iter().enumerate().take(limit) {
            #[allow(clippy::cast_possible_truncation)]
            pos_map[code.0 as usize].push(i as u16);
        }

        let fp_rubric = FixedPointRubric::from(rubric);
        let fp_keys: Vec<FixedPointKey> = keyboard.keys.iter().map(FixedPointKey::from).collect();

        // 1. Monograms: Optimal Choice
        total_score = total_score.saturating_add(Self::score_monograms(
            keyboard, corpus, cost_model, &pos_map,
        ));

        // 2. Bigrams: Optimal Choice
        total_score = total_score.saturating_add(Self::score_bigrams(
            keyboard, corpus, &fp_rubric, &fp_keys, &pos_map, cost_model,
        ));

        // 3. Trigrams: Optimal Choice
        total_score = total_score
            .saturating_add(Self::score_trigrams(corpus, &fp_rubric, &fp_keys, &pos_map));

        total_score
    }

    #[allow(clippy::cast_possible_wrap)]
    fn score_monograms(
        keyboard: &Keyboard,
        corpus: &Corpus,
        cost_model: &CostModel,
        pos_map: &[Vec<u16>],
    ) -> i64 {
        let mut total_score: i64 = 0;
        let model_key = "model_a_row_staggered";

        let empty_map = std::collections::HashMap::new();
        let static_costs = match cost_model.models.get(model_key) {
            Some(m) => &m.static_costs,
            None => &empty_map,
        };

        for (char_code, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            let candidates = &pos_map[char_code];
            if candidates.is_empty() {
                continue;
            }

            let mut min_cost = i64::MAX;
            for &p_idx in candidates {
                let k = &keyboard.keys[p_idx as usize];

                // Static key cost from model
                let cost = resolve_key_cost(k, static_costs);
                let cost_fixed = to_fixed(cost);
                if cost_fixed < min_cost {
                    min_cost = cost_fixed;
                }
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
        cost_model: &CostModel,
    ) -> i64 {
        let mut total_score: i64 = 0;

        let mut sequence_modifiers = std::collections::HashMap::new();
        for (bigram, &val) in &cost_model.dynamic_rules.sequence_modifiers {
            if bigram.len() == 2 {
                let bytes = bigram.as_bytes();
                let key = (u16::from(bytes[0]), u16::from(bytes[1]));
                sequence_modifiers.insert(key, to_fixed(val));
            }
        }

        for &(c1, c2, freq) in &corpus.bigrams {
            let candidates1 = &pos_map[c1 as usize];
            let candidates2 = &pos_map[c2 as usize];
            if candidates1.is_empty() || candidates2.is_empty() {
                continue;
            }

            let mut min_cost = i64::MAX;
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    let k1 = &fp_keys[p1 as usize];
                    let k2 = &fp_keys[p2 as usize];
                    let mut cost =
                        fp_rubric.calculate_pair_cost(keyboard, k1, k2, p1 as usize, p2 as usize);

                    if let Some(&mod_val) = sequence_modifiers.get(&(c1, c2)) {
                        cost = cost.saturating_add(mod_val);
                    }

                    if cost < min_cost {
                        min_cost = cost;
                    }
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
            if candidates1.is_empty() || candidates2.is_empty() || candidates3.is_empty() {
                continue;
            }

            let mut min_cost = i64::MAX;
            for &p1 in candidates1 {
                for &p2 in candidates2 {
                    for &p3 in candidates3 {
                        let k1 = &fp_keys[p1 as usize];
                        let k2 = &fp_keys[p2 as usize];
                        let k3 = &fp_keys[p3 as usize];
                        let cost = fp_rubric.calculate_flow_cost(k1, k2, k3);
                        if cost < min_cost {
                            min_cost = cost;
                        }
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

fn resolve_key_cost(
    key: &KeyNode,
    static_costs: &std::collections::HashMap<String, HandDefinition>,
) -> f32 {
    let hand_key = if key.hand == HandIndex::LEFT {
        "left_hand"
    } else {
        "right_hand"
    };
    let hand_def = static_costs
        .get(hand_key)
        .or_else(|| static_costs.get("universal_hand"));

    if let Some(hand) = hand_def {
        let finger_key = match key.finger {
            FingerIndex::THUMB => "thumb",
            FingerIndex::INDEX => "index",
            FingerIndex::MIDDLE => "middle",
            FingerIndex::RING => "ring",
            FingerIndex::PINKY => "pinky",
            _ => "unknown",
        };

        if let Some(finger_def) = hand.fingers.get(finger_key) {
            match finger_def {
                FingerDefinition::Standard(zones) => {
                    let zone_key = if key.col.0.abs() > 1 && key.finger == FingerIndex::INDEX {
                        "inner"
                    } else if key.col.0.abs() > 1 && key.finger == FingerIndex::PINKY {
                        "outer"
                    } else {
                        "base"
                    };

                    if let Some(zone) = zones.get(zone_key).or_else(|| zones.get("base")) {
                        let row_key = format!("r{}", key.row.0);
                        if let Some(cost) = zone.get(&row_key) {
                            return *cost;
                        }
                    }
                }
                FingerDefinition::Thumb(positions) => {
                    return *positions
                        .values()
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(&100.0);
                }
            }
        }
    }
    100.0
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn to_fixed(val: f32) -> i64 {
    if val.is_nan() {
        return 0;
    }
    if val.is_infinite() {
        return if val.is_sign_positive() {
            i64::MAX
        } else {
            i64::MIN
        };
    }
    let scaled = f64::from(val) * f64::from(SCORE_SCALE);
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
    ) -> i64 {
        if idx1 == idx2 {
            return 0;
        }

        // Travel Distance (Weighted Squared Euclidean)
        // Task-phys-031: Use the fixed-point coordinates stored in the key
        let dx = (to_f32(k1.x) - to_f32(k2.x)).abs();
        let dy = (to_f32(k1.y) - to_f32(k2.y)).abs();
        let dist_raw = (dx * dx * to_f32(self.travel_lat)) + (dy * dy * to_f32(self.travel_vert));
        let dist_cost = to_fixed(dist_raw);

        if k1.hand != k2.hand {
            return 0;
        }

        let mut cost = dist_cost;

        if k1.finger == k2.finger {
            let mut reach_k2 = 0i64;
            if let Some(origin) = kb
                .finger_origins
                .get(k2.hand.as_usize())
                .and_then(|h| h.get(k2.finger.as_usize()))
            {
                let rdx = (to_f32(k2.x) - origin.0).abs();
                let rdy = (to_f32(k2.y) - origin.1).abs();
                let reach_raw =
                    (rdx * rdx * to_f32(self.travel_lat)) + (rdy * rdy * to_f32(self.travel_vert));
                reach_k2 = to_fixed(reach_raw);
            }

            cost = dist_cost.saturating_sub(reach_k2);

            let row_diff = (k1.row.0 - k2.row.0).abs();
            let col_diff = (k1.col.0 - k2.col.0).abs();

            if col_diff == 1 {
                if k1.finger.is_weak() {
                    cost = cost.saturating_add(self.sfb_lateral_weak);
                } else {
                    cost = cost.saturating_add(self.sfb_lateral);
                }
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
                if col_dist > 1 {
                    cost = cost.saturating_add(self.sfb_lateral);
                }
            }
        }
        cost
    }

    fn calculate_flow_cost(
        &self,
        k1: &FixedPointKey,
        k2: &FixedPointKey,
        k3: &FixedPointKey,
    ) -> i64 {
        if k1.hand != k2.hand || k2.hand != k3.hand {
            return 0;
        }
        if k1.finger == k3.finger && k1.finger != k2.finger {
            return self.redirect;
        }
        let dir1 = k2.finger.diff(k1.finger);
        let dir2 = k3.finger.diff(k2.finger);
        if dir1 == 0 || dir2 == 0 {
            return 0;
        }
        if dir1.signum() != dir2.signum() {
            return self.redirect;
        }
        if dir1 < 0 {
            return self.roll_bonus.saturating_neg();
        }
        0
    }
}

#[allow(clippy::cast_precision_loss)]
fn to_f32(val: i64) -> f32 {
    val as f32 / SCORE_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScoringEngine;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, KeyCode, RowIndex};
    use keyforge_model::{Corpus, CostModel, KeyNode, Keyboard, Layout, Rubric};
    use proptest::prelude::*;
    use std::sync::Arc;

    fn load_cost_model_fixture() -> CostModel {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/default_cost_model.json");
        let json = std::fs::read_to_string(path).expect("Failed to read fixture");
        serde_json::from_str(&json).expect("Failed to parse fixture")
    }

    fn rubric_strategy() -> impl Strategy<Value = Rubric> {
        (
            0.0..1000.0f32,
            0.0..500.0f32,
            0.0..10.0f32,
            0.0..5.0f32,
            prop::array::uniform5(0.0..5.0f32),
            0.0..200.0f32,
            0.0..100.0f32,
        )
            .prop_map(
                |(sfb, sfb_lat, t_lat, t_vert, fingers, redir, roll)| Rubric {
                    sfb_base: sfb,
                    sfb_lateral: sfb_lat,
                    travel_lat: t_lat,
                    travel_vert: t_vert,
                    finger_effort: fingers,
                    redirect: redir,
                    roll_bonus: roll,
                    trigram_coverage: 1.0,
                    trigram_limit: 100_000,
                    ..Default::default()
                },
            )
    }

    fn kb_and_layout_strategy() -> impl Strategy<Value = (Keyboard, Vec<KeyCode>)> {
        (10..50usize).prop_flat_map(|count| {
            let kb_strat = prop::collection::vec(
                (
                    -20.0..20.0f32,
                    -20.0..20.0f32,
                    0u8..2,
                    0u8..5,
                    -5i8..5,
                    -10i8..15,
                ),
                count,
            )
            .prop_map(move |keys_data| {
                let keys = keys_data
                    .into_iter()
                    .enumerate()
                    .map(|(i, (x, y, hand, finger, row, col))| KeyNode {
                        index: i,
                        label: format!("k{}", i),
                        hand: HandIndex(hand),
                        finger: FingerIndex(finger),
                        row: RowIndex(row),
                        col: ColIndex(col),
                        x,
                        y,
                        is_home: row == 1,
                        ..Default::default()
                    })
                    .collect();
                Keyboard::new(keys, 1).unwrap()
            });

            let layout_strat = prop::collection::vec(0u16..255, count)
                .prop_map(|codes| codes.into_iter().map(KeyCode).collect::<Vec<_>>());

            (kb_strat, layout_strat)
        })
    }

    fn corpus_strategy(char_range: std::ops::Range<u16>) -> impl Strategy<Value = Corpus> {
        (
            prop::collection::vec((char_range.clone(), char_range.clone(), 1u32..1000), 0..20),
            prop::collection::vec(
                (
                    char_range.clone(),
                    char_range.clone(),
                    char_range.clone(),
                    1u32..1000,
                ),
                0..20,
            ),
            prop::collection::vec(0u64..1000, 256),
        )
            .prop_map(|(bigrams, trigrams, char_freqs)| {
                let mut c = Corpus::default();
                c.bigrams = bigrams;
                c.trigrams = trigrams;
                c.char_freqs = char_freqs;
                c
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_oracle_parity(
            (kb, layout_keys) in kb_and_layout_strategy(),
            corpus in corpus_strategy(0..255),
            rubric in rubric_strategy()
        ) {
            let layout = Layout::new_unchecked(layout_keys);
            let cost_model = load_cost_model_fixture();
            let engine = ScoringEngine::new(&kb, &corpus, &rubric, &cost_model).unwrap();

            let fast_score = engine.score(&layout).unwrap();
            let reference_score = DeterministicScorer::score(&kb, &corpus, &rubric, &layout, &cost_model);

            // Increased tolerance for f32 accumulation differences
            let tolerance = 1.0;
            prop_assert!((fast_score - reference_score).abs() < tolerance,
                "Score mismatch! Fast: {}, Ref: {}", fast_score, reference_score);
        }
    }
}
