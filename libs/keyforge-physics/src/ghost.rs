// libs/keyforge-physics/src/ghost.rs

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

//! # Ghost Model
//!
//! Mandated by the KeyForge Engineering Manifesto, the Ghost Model provides
//! a "Simple, Slow, Correct" reference implementation of the scoring logic.
//!
//! This module avoids all performance optimizations (SIMD, cache blocking,
//! CSR-flattening) in favor of maximum readability and mathematical clarity.

use crate::error::PhysicsError;
use keyforge_model::{Corpus, KeyCode, Keyboard, Layout, Rubric, Score};

/// Reference scorer implementing the "Ghost Model" pattern.
/// Use this to verify optimized kernels.
#[derive(Debug)]
pub struct GhostScorer {
    keyboard: Keyboard,
    rubric: GhostRubric,
    cost_model: keyforge_model::CostModel,
}

impl GhostScorer {
    /// Creates a new `GhostScorer`.
    ///
    /// # Errors
    /// Returns `PhysicsError::Config` if the rubric contains invalid values.
    pub fn new(
        kb: Keyboard,
        rubric: &Rubric,
        cm: keyforge_model::CostModel,
    ) -> Result<Self, PhysicsError> {
        Ok(Self {
            keyboard: kb,
            rubric: GhostRubric::from_rubric(rubric),
            cost_model: cm,
        })
    }

    /// Pure reference scoring algorithm.
    ///
    /// # Errors
    /// Returns `PhysicsError::ScoreOverflow` if arithmetic fails.
    pub fn score(&self, corpus: &Corpus, layout: &Layout) -> Result<Score, PhysicsError> {
        let mut total = Score::ZERO;

        // 1. Monograms (Static Effort)
        for (code_val, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            let code = KeyCode::new(code_val.try_into().unwrap_or_default());
            if code == KeyCode::EMPTY || code == KeyCode::TRANSPARENT {
                continue;
            }

            let min_cost = self.find_min_monogram_cost(layout, code)?;
            let freq_i64: i64 = freq.try_into().unwrap_or_default();
            let contrib = min_cost
                .checked_mul(freq_i64)
                .ok_or(PhysicsError::ScoreOverflow {
                    context: format!("Ghost Monogram overflow for keycode {code}"),
                })?;
            total = total
                .checked_add(contrib)
                .ok_or(PhysicsError::ScoreOverflow {
                    context: format!("Ghost Monogram total overflow at keycode {code}"),
                })?;
        }

        // 2. Bigrams (Movement)

        for (c1, c2, freq) in &*corpus.bigrams {
            let code1 = KeyCode::new(*c1);

            let code2 = KeyCode::new(*c2);

            if code1 == KeyCode::EMPTY || code2 == KeyCode::EMPTY {
                continue;
            }

            let min_cost = self.find_min_bigram_cost(layout, code1, code2);

            let freq_i64: i64 = (*freq).into();

            let contrib = min_cost
                .checked_mul(freq_i64)
                .ok_or(PhysicsError::ScoreOverflow {
                    context: format!("Ghost Bigram overflow for ({code1}, {code2})"),
                })?;

            total = total
                .checked_add(contrib)
                .ok_or(PhysicsError::ScoreOverflow {
                    context: format!("Ghost Bigram total overflow at ({code1}, {code2})"),
                })?;
        }

        // 3. Trigrams (Flow)

        for (c1, c2, c3, freq) in &*corpus.trigrams {
            let code1 = KeyCode::new(*c1);

            let code2 = KeyCode::new(*c2);

            let code3 = KeyCode::new(*c3);

            if code1 == KeyCode::EMPTY || code2 == KeyCode::EMPTY || code3 == KeyCode::EMPTY {
                continue;
            }

            let min_cost = self.find_min_trigram_cost(layout, code1, code2, code3)?;

            let freq_i64: i64 = (*freq).into();

            let contrib = min_cost
                .checked_mul(freq_i64)
                .ok_or(PhysicsError::ScoreOverflow {
                    context: format!("Ghost Trigram overflow for ({code1}, {code2}, {code3})"),
                })?;

            total = total
                .checked_add(contrib)
                .ok_or(PhysicsError::ScoreOverflow {
                    context: format!("Ghost Trigram total overflow at ({code1}, {code2}, {code3})"),
                })?;
        }

        Ok(total)
    }

    fn find_min_monogram_cost(
        &self,

        layout: &Layout,

        code: KeyCode,
    ) -> Result<Score, PhysicsError> {
        let mut min = Score::MAX;

        let positions = self.find_all_positions(layout, code);

        for pos in positions {
            let key = &self.keyboard.keys()[pos];

            let effort = self.rubric.finger_effort[key.finger.as_usize()];

            // In Ghost mode, we don't cache, we look up every time

            let static_cost = self.resolve_static_cost(key)?;

            let total = effort + static_cost;

            if total < min {
                min = total;
            }
        }

        Ok(min)
    }

    fn find_min_bigram_cost(&self, layout: &Layout, c1: KeyCode, c2: KeyCode) -> Score {
        let mut min = Score::MAX;

        let pos1 = self.find_all_positions(layout, c1);

        let pos2 = self.find_all_positions(layout, c2);

        for p1 in pos1 {
            for p2 in &pos2 {
                let cost = self.calculate_pair_cost(p1, *p2);

                if cost < min {
                    min = cost;
                }
            }
        }

        min
    }

    #[allow(clippy::unnecessary_wraps)]
    fn find_min_trigram_cost(
        &self,
        layout: &Layout,
        c1: KeyCode,
        c2: KeyCode,
        c3: KeyCode,
    ) -> Result<Score, PhysicsError> {
        let mut min = Score::MAX;
        let pos1 = self.find_all_positions(layout, c1);
        let pos2 = self.find_all_positions(layout, c2);
        let pos3 = self.find_all_positions(layout, c3);

        for p1 in pos1 {
            for p2 in &pos2 {
                for p3 in &pos3 {
                    let cost = self.calculate_flow_cost(p1, *p2, *p3);
                    if cost < min {
                        min = cost;
                    }
                }
            }
        }
        Ok(min)
    }

    #[allow(clippy::unused_self)]
    fn find_all_positions(&self, layout: &Layout, code: KeyCode) -> Vec<usize> {
        layout
            .keys()
            .iter()
            .enumerate()
            .filter(|(_, &k)| k == code)
            .map(|(i, _)| i)
            .collect()
    }

    fn resolve_static_cost(&self, key: &keyforge_model::KeyNode) -> Result<Score, PhysicsError> {
        // Ghost implementation of static cost resolution
        // Simplified version of verify.rs logic
        let model_key = if self.keyboard.kb_type.to_lowercase().contains("ortho") {
            "model_ortho"
        } else {
            "model_a_row_staggered"
        };

        let hand_key = if key.hand.is_left() {
            "left_hand"
        } else {
            "right_hand"
        };

        let val = self
            .cost_model
            .models()
            .get(model_key)
            .and_then(|m| {
                m.static_costs
                    .get(hand_key)
                    .or_else(|| m.static_costs.get("universal_hand"))
            })
            .and_then(|h| {
                let f_key = match key.finger {
                    keyforge_model::FingerIndex::THUMB => "thumb",
                    keyforge_model::FingerIndex::INDEX => "index",
                    keyforge_model::FingerIndex::MIDDLE => "middle",
                    keyforge_model::FingerIndex::RING => "ring",
                    keyforge_model::FingerIndex::PINKY => "pinky",
                    _ => "unknown",
                };
                h.fingers.get(f_key)
            })
            .map(|f| match f {
                keyforge_model::cost_model::FingerDefinition::Standard(reach) => {
                    reach.base.get(&key.row).copied().unwrap_or(Score::ZERO)
                }
                keyforge_model::cost_model::FingerDefinition::Thumb(pos) => {
                    pos.values().min().copied().unwrap_or(Score::ZERO)
                }
                keyforge_model::cost_model::FingerDefinition::Fallback(_) => Score::ZERO,
            })
            .ok_or_else(|| {
                PhysicsError::Config(format!(
                    "Could not resolve static cost for key at index {}",
                    key.index
                ))
            })?;

        Ok(val)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_pair_cost(&self, p1: usize, p2: usize) -> Score {
        let k1 = &self.keyboard.keys()[p1];
        let k2 = &self.keyboard.keys()[p2];
        if k1.hand != k2.hand {
            return Score::ZERO;
        }

        let movement = self.keyboard.spatial_cache[p1 * self.keyboard.keys().len() + p2];
        let dx2 = i64::from(movement.dx) * i64::from(movement.dx);
        let dy2 = i64::from(movement.dy) * i64::from(movement.dy);

        let t_lat_i = i128::from(self.rubric.travel_lat.raw());
        let t_vert_i = i128::from(self.rubric.travel_vert.raw());
        let dist_sq_weighted = i128::from(dx2) * t_lat_i + i128::from(dy2) * t_vert_i;
        let mut cost = Score::from_scaled_i64(crate::kernel::mechanics::integer_sqrt_i128(
            dist_sq_weighted,
        ));

        if k1.finger == k2.finger {
            // SFB Handling
            cost = cost.checked_add(self.rubric.sfb_base).unwrap_or(Score::MAX);
        }

        cost
    }

    fn calculate_flow_cost(&self, p1: usize, p2: usize, p3: usize) -> Score {
        let k1 = &self.keyboard.keys()[p1];
        let k2 = &self.keyboard.keys()[p2];
        let k3 = &self.keyboard.keys()[p3];

        crate::kernel::mechanics::calculate_flow_cost(
            k1.hand,
            k2.hand,
            k3.hand,
            k1.finger,
            k2.finger,
            k3.finger,
            self.rubric.redirect,
            self.rubric.roll_bonus,
            self.rubric.roll_out_bonus,
        )
    }
}

#[derive(Debug)]
struct GhostRubric {
    finger_effort: Vec<Score>,
    travel_lat: Score,
    travel_vert: Score,
    sfb_base: Score,
    redirect: Score,
    roll_bonus: Score,
    roll_out_bonus: Score,
}

impl GhostRubric {
    fn from_rubric(r: &Rubric) -> Self {
        let finger_effort = r.finger_effort().to_vec();

        Self {
            finger_effort,

            travel_lat: r.travel_lat(),

            travel_vert: r.travel_vert(),

            sfb_base: r.sfb_base(),

            redirect: r.redirect(),

            roll_bonus: r.roll_bonus(),

            roll_out_bonus: r.roll_out_bonus(),
        }
    }
}
