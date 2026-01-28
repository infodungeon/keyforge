// libs/keyforge-physics/src/verify.rs

use crate::PhysicsError;
use keyforge_model::{types::KeyCode, Corpus, KeyNode, Keyboard, Rubric};
use std::sync::Arc;

/// A naive, high-precision version of the scoring logic used to verify
/// the optimized `ScoringEngine` results.
#[derive(Debug, Clone)]
pub struct DeterministicScorer {
    rubric: FixedPointRubric,
    spatial_index: keyforge_model::keyboard::SpatialIndex,
    static_costs:
        Arc<std::collections::HashMap<String, keyforge_model::cost_model::HandDefinition>>,
    sequence_modifiers: Arc<std::collections::HashMap<(u16, u16), i64>>,
    penalty_redirect: i64,
    bonus_roll: i64,
    bonus_roll_out: i64,
}

impl DeterministicScorer {
    /// Creates a new `DeterministicScorer` from keyboard, rubric and cost model.
    #[must_use]
    pub fn new(kb: &Keyboard, rubric: &Rubric, cost_model: &keyforge_model::CostModel) -> Self {
        let model_key = if kb.kb_type.to_lowercase().contains("ortho") {
            "model_ortho"
        } else {
            "model_a_row_staggered"
        };
        let static_costs = cost_model
            .models()
            .get(model_key)
            .map(|m| Arc::new(m.static_costs.clone()))
            .unwrap_or_default();

        let mut sequence_modifiers = std::collections::HashMap::new();
        for (bigram, &val) in &cost_model.dynamic_rules().sequence_modifiers.map {
            if bigram.len() == 2 {
                let bytes = bigram.as_bytes();
                let key = (u16::from(bytes[0]), u16::from(bytes[1]));
                sequence_modifiers.insert(key, to_fixed(val));
            }
        }

        let spatial_index = keyforge_model::keyboard::SpatialIndex::build_from(kb);

        Self {
            rubric: FixedPointRubric::from_rubric(rubric),
            spatial_index,
            static_costs,
            sequence_modifiers: Arc::new(sequence_modifiers),
            penalty_redirect: to_fixed(rubric.redirect()),
            bonus_roll: to_fixed(rubric.roll_bonus()),
            bonus_roll_out: to_fixed(rubric.roll_out_bonus()),
        }
    }

    /// Scores a layout bit-for-bit against the naive algorithm.
    ///
    /// # Errors
    /// Returns `PhysicsError` if static costs cannot be resolved or if a score overflows `i64`.
    pub fn score_detailed(
        &self,
        kb: &Keyboard,
        corpus: &Corpus,
        layout_keys: &[KeyCode],
    ) -> Result<(i64, i64, i64), PhysicsError> {
        let mono_score = self.score_monograms(kb, corpus, layout_keys)?;
        let bigram_score = self.score_bigrams(kb, corpus, layout_keys)?;
        let trigram_score = self.score_trigrams(kb, corpus, layout_keys)?;

        Ok((mono_score, bigram_score, trigram_score))
    }

    fn score_monograms(
        &self,
        kb: &Keyboard,
        corpus: &Corpus,
        layout_keys: &[KeyCode],
    ) -> Result<i64, PhysicsError> {
        let mut mono_score = 0i64;
        for (code_val, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            let code = KeyCode(code_val.try_into().unwrap_or_default());
            let indices = find_indices(layout_keys, code);
            if indices.is_empty() {
                continue;
            }

            let mut min_total_cost = i64::MAX;
            for &idx in &indices {
                let key = &kb.keys[idx];
                let effort = self.rubric.finger_effort[key.finger.as_usize()];
                let static_cost = to_fixed(resolve_static_key_cost(key, &self.static_costs)?);
                let total =
                    effort
                        .checked_add(static_cost)
                        .ok_or_else(|| PhysicsError::ScoreOverflow {
                            context: format!("Monogram effort + static cost for code {code_val}"),
                        })?;
                if total < min_total_cost {
                    min_total_cost = total;
                }
            }
            let freq_i64: i64 = freq.try_into().unwrap_or_default();
            let contrib = min_total_cost.checked_mul(freq_i64).ok_or_else(|| {
                PhysicsError::ScoreOverflow {
                    context: format!("Monogram freq scale for code {code_val}"),
                }
            })?;
            mono_score =
                mono_score
                    .checked_add(contrib)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: format!("Monogram total accumulation at code {code_val}"),
                    })?;
        }
        Ok(mono_score)
    }

    fn score_bigrams(
        &self,
        kb: &Keyboard,
        corpus: &Corpus,
        layout_keys: &[KeyCode],
    ) -> Result<i64, PhysicsError> {
        let mut bigram_score = 0i64;
        for (c1, c2, freq) in &*corpus.bigrams.entries {
            let freq = i64::from(*freq);
            let indices1 = find_indices(layout_keys, KeyCode(*c1));
            let indices2 = find_indices(layout_keys, KeyCode(*c2));

            if !indices1.is_empty() && !indices2.is_empty() {
                let mut min_cost = i64::MAX;
                for &idx1 in &indices1 {
                    for &idx2 in &indices2 {
                        let mut cost = self.rubric.calculate_pair_cost(
                            kb,
                            &self.spatial_index,
                            &kb.keys[idx1],
                            &kb.keys[idx2],
                            idx1,
                            idx2,
                        )?;

                        // Apply sequence modifiers
                        if let Some(&mod_val) = self.sequence_modifiers.get(&(*c1, *c2)) {
                            cost = cost.checked_add(mod_val).ok_or_else(|| {
                                PhysicsError::ScoreOverflow {
                                    context: format!("Bigram modifier for ({c1}, {c2})"),
                                }
                            })?;
                        }

                        if cost < min_cost {
                            min_cost = cost;
                        }
                    }
                }
                if min_cost != i64::MAX {
                    let contrib =
                        min_cost
                            .checked_mul(freq)
                            .ok_or_else(|| PhysicsError::ScoreOverflow {
                                context: format!("Bigram freq scale for ({c1}, {c2})"),
                            })?;
                    bigram_score = bigram_score.checked_add(contrib).ok_or_else(|| {
                        PhysicsError::ScoreOverflow {
                            context: format!("Bigram total accumulation at ({c1}, {c2})"),
                        }
                    })?;
                }
            }
        }
        Ok(bigram_score)
    }

    fn score_trigrams(
        &self,
        kb: &Keyboard,
        corpus: &Corpus,
        layout_keys: &[KeyCode],
    ) -> Result<i64, PhysicsError> {
        let mut trigram_score = 0i64;
        for (c1, c2, c3, freq) in &*corpus.trigrams.entries {
            let freq = i64::from(*freq);
            let indices1 = find_indices(layout_keys, KeyCode(*c1));
            let indices2 = find_indices(layout_keys, KeyCode(*c2));
            let indices3 = find_indices(layout_keys, KeyCode(*c3));

            if !indices1.is_empty() && !indices2.is_empty() && !indices3.is_empty() {
                let mut min_cost = i64::MAX;
                for &idx1 in &indices1 {
                    for &idx2 in &indices2 {
                        for &idx3 in &indices3 {
                            let cost = self.calculate_flow_cost(kb, idx1, idx2, idx3);
                            if cost < min_cost {
                                min_cost = cost;
                            }
                        }
                    }
                }
                if min_cost != i64::MAX {
                    let contrib =
                        min_cost
                            .checked_mul(freq)
                            .ok_or_else(|| PhysicsError::ScoreOverflow {
                                context: format!("Trigram freq scale for ({c1}, {c2}, {c3})"),
                            })?;
                    trigram_score = trigram_score.checked_add(contrib).ok_or_else(|| {
                        PhysicsError::ScoreOverflow {
                            context: format!("Trigram total accumulation at ({c1}, {c2}, {c3})"),
                        }
                    })?;
                }
            }
        }
        Ok(trigram_score)
    }

    /// Combined total score.
    ///
    /// # Errors
    /// Returns `PhysicsError` if scoring fails or overflows.
    pub fn score(
        &self,
        kb: &Keyboard,
        corpus: &Corpus,
        layout_keys: &[KeyCode],
    ) -> Result<i64, PhysicsError> {
        let (m, b, t) = self.score_detailed(kb, corpus, layout_keys)?;
        m.checked_add(b)
            .and_then(|sum| sum.checked_add(t))
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Final oracle total accumulation".into(),
            })
    }

    fn calculate_flow_cost(&self, kb: &Keyboard, p1: usize, p2: usize, p3: usize) -> i64 {
        let k1 = &kb.keys[p1];
        let k2 = &kb.keys[p2];
        let k3 = &kb.keys[p3];

        crate::kernel::mechanics::calculate_flow_cost(
            k1.hand,
            k2.hand,
            k3.hand,
            k1.finger,
            k2.finger,
            k3.finger,
            keyforge_model::types::Score(self.penalty_redirect),
            keyforge_model::types::Score(self.bonus_roll),
            keyforge_model::types::Score(self.bonus_roll_out),
        )
        .0
    }
}

fn find_indices(layout: &[KeyCode], target: KeyCode) -> Vec<usize> {
    layout
        .iter()
        .enumerate()
        .filter(|(_, &k)| k == target)
        .map(|(i, _)| i)
        .collect()
}

fn resolve_static_key_cost(
    key: &KeyNode,
    static_costs: &std::collections::HashMap<String, keyforge_model::cost_model::HandDefinition>,
) -> Result<f32, PhysicsError> {
    let hand_key = if key.hand == keyforge_model::HandIndex::LEFT {
        "left_hand"
    } else {
        "right_hand"
    };
    let hand_def = static_costs
        .get(hand_key)
        .or_else(|| static_costs.get("universal_hand"));

    if let Some(hand) = hand_def {
        let finger_key = match key.finger {
            keyforge_model::FingerIndex::THUMB => "thumb",
            keyforge_model::FingerIndex::INDEX => "index",
            keyforge_model::FingerIndex::MIDDLE => "middle",
            keyforge_model::FingerIndex::RING => "ring",
            keyforge_model::FingerIndex::PINKY => "pinky",
            _ => "unknown",
        };

        if let Some(finger_def) = hand.fingers.get(finger_key) {
            use keyforge_model::cost_model::FingerDefinition;
            match finger_def {
                FingerDefinition::Standard(zones) => {
                    let zone = if key.col.0.unsigned_abs() > 1
                        && key.finger == keyforge_model::FingerIndex::INDEX
                    {
                        &zones.inner
                    } else if key.col.0.unsigned_abs() > 1
                        && key.finger == keyforge_model::FingerIndex::PINKY
                    {
                        &zones.outer
                    } else {
                        &zones.base
                    };

                    let target_zone = if zone.is_empty() { &zones.base } else { zone };
                    return Ok(target_zone.get(&key.row).copied().unwrap_or(0.0));
                }
                FingerDefinition::Thumb(positions) => {
                    return Ok(positions
                        .values()
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .copied()
                        .unwrap_or(0.0));
                }
            }
        }
    }

    Err(PhysicsError::Config(format!(
        "Finger {:?} not found in hand {} or universal_hand",
        key.finger, hand_key
    )))
}

#[derive(Debug, Clone)]
struct FixedPointRubric {
    finger_effort: Vec<i64>,
    travel_lat: i64,
    travel_vert: i64,
    sfb_base: i64,
    sfb_lateral: i64,
    sfb_lateral_weak: i64,
    sfb_diagonal: i64,
    sfb_long: i64,
    penalty_scissor: i64,
    threshold_sfb_long_row_diff: i32,
    threshold_scissor_row_diff: i32,
}

impl FixedPointRubric {
    fn from_rubric(r: &Rubric) -> Self {
        Self {
            finger_effort: r.finger_effort().iter().map(|&e| to_fixed(e)).collect(),
            travel_lat: to_fixed(r.travel_lat()),
            travel_vert: to_fixed(r.travel_vert()),
            sfb_base: to_fixed(r.sfb_base()),
            sfb_lateral: to_fixed(r.sfb_lateral()),
            sfb_lateral_weak: to_fixed(r.sfb_lateral_weak()),
            sfb_diagonal: to_fixed(r.sfb_diagonal()),
            sfb_long: to_fixed(r.sfb_long()),
            penalty_scissor: to_fixed(r.penalty_scissor()),
            threshold_sfb_long_row_diff: i32::from(r.threshold_sfb_long_row_diff()),
            threshold_scissor_row_diff: i32::from(r.threshold_scissor_row_diff()),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_pair_cost(
        &self,
        kb: &Keyboard,
        spatial_index: &keyforge_model::keyboard::SpatialIndex,
        k1: &KeyNode,
        k2: &KeyNode,
        idx1: usize,
        idx2: usize,
    ) -> Result<i64, PhysicsError> {
        if idx1 == idx2 || k1.hand != k2.hand {
            return Ok(0);
        }

        let (dx2, dy2) = spatial_index.spatial_cache[idx1 * kb.keys.len() + idx2];
        let t_lat = f64::from(to_f32(self.travel_lat));
        let t_vert = f64::from(to_f32(self.travel_vert));
        let scale = f64::from(keyforge_model::constants::SCORE_SCALE);

        let dist_raw = ((f64::from(dx2) * t_lat) + (f64::from(dy2) * t_vert)) * scale;
        if dist_raw.is_nan() || dist_raw.is_infinite() {
            return Err(PhysicsError::InvalidInput {
                message: format!("Oracle geometric distance between keys {idx1} and {idx2} is invalid (NaN or Infinite)")
            });
        }

        let mut cost = dist_raw.round() as i64;
        if k1.finger == k2.finger {
            cost =
                self.apply_sfb_oracle_logic(spatial_index, k1, k2, cost, t_lat, t_vert, scale)?;
        } else {
            cost = self.apply_non_sfb_oracle_logic(k1, k2, cost);
        }

        Ok(cost)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn apply_sfb_oracle_logic(
        &self,
        spatial_index: &keyforge_model::keyboard::SpatialIndex,
        k1: &KeyNode,
        k2: &KeyNode,
        mut cost: i64,
        t_lat: f64,
        t_vert: f64,
        scale: f64,
    ) -> Result<i64, PhysicsError> {
        let mut reach_k2 = 0.0f64;
        if let Some(origin) = spatial_index
            .finger_origins
            .get(k2.hand.as_usize())
            .and_then(|h| h.get(k2.finger.as_usize()))
        {
            let rdx = f64::from(k2.x - origin.0);
            let rdy = f64::from(k2.y - origin.1);
            reach_k2 = ((rdx * rdx * t_lat) + (rdy * rdy * t_vert)) * scale;
        }

        cost = cost.checked_sub(reach_k2.round() as i64).ok_or_else(|| {
            PhysicsError::ScoreOverflow {
                context: "Oracle SFB reach reduction".to_string(),
            }
        })?;

        let row_diff = (i32::from(k1.row.0) - i32::from(k2.row.0)).unsigned_abs();
        let col_diff = (i32::from(k1.col.0) - i32::from(k2.col.0)).unsigned_abs();

        if col_diff == 1 {
            let sfb_extra = if k1.finger.is_weak() {
                self.sfb_lateral_weak
            } else {
                self.sfb_lateral
            };
            cost = cost
                .checked_add(sfb_extra)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB lateral".to_string(),
                })?;
        } else if col_diff > 1 {
            cost =
                cost.checked_add(self.sfb_diagonal)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "Oracle SFB diagonal".to_string(),
                    })?;
        } else if row_diff
            >= self
                .threshold_sfb_long_row_diff
                .try_into()
                .unwrap_or_default()
        {
            cost = cost
                .checked_add(self.sfb_long)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB long".to_string(),
                })?;
        } else {
            cost = cost
                .checked_add(self.sfb_base)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB base".to_string(),
                })?;
        }
        Ok(cost)
    }

    fn apply_non_sfb_oracle_logic(&self, k1: &KeyNode, k2: &KeyNode, mut cost: i64) -> i64 {
        let finger_diff = k1.finger.distance(k2.finger);
        let row_diff = (i32::from(k1.row.0) - i32::from(k2.row.0)).unsigned_abs();

        if finger_diff == 1
            && k1.finger != keyforge_model::types::FingerIndex::THUMB
            && k2.finger != keyforge_model::types::FingerIndex::THUMB
        {
            if row_diff
                >= self
                    .threshold_scissor_row_diff
                    .try_into()
                    .unwrap_or_default()
            {
                cost = cost.saturating_add(self.penalty_scissor);
            } else if row_diff == 0 {
                let col_diff = (i32::from(k1.col.0) - i32::from(k2.col.0)).unsigned_abs();
                if col_diff > 1 {
                    cost = cost.saturating_add(self.sfb_lateral);
                }
            }
        }
        cost
    }
}

/// Converts a float to fixed-point integer ticks.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn to_fixed(f: f32) -> i64 {
    // Intentional truncation: converting float score to fixed-point integer ticks.
    (f * keyforge_model::constants::SCORE_SCALE) as i64
}

/// Converts fixed-point integer ticks back to a float.
#[allow(clippy::cast_precision_loss)]
fn to_f32(i: i64) -> f32 {
    // Precision loss acceptable for display/API values.
    (i as f32) / keyforge_model::constants::SCORE_SCALE
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::testing::setup_minimal_assets;
    use keyforge_model::Layout;

    #[test]
    fn test_oracle_parity() {
        let (kb, cp, rb, cm) = setup_minimal_assets();
        let scorer = DeterministicScorer::new(&kb, &rb, &cm);

        let layout_keys = vec![KeyCode(97), KeyCode(98), KeyCode(99)];
        let layout = Layout::new_unchecked(layout_keys.clone());

        let oracle_score = scorer.score(&kb, &cp, &layout_keys).unwrap();

        // Compare with generic engine
        let engine = crate::EngineFactory::new_generic(&crate::ScoringContext::new(
            Arc::new(kb),
            Arc::new(cp),
            Arc::new(rb),
            Arc::new(cm),
            keyforge_model::config::EngineConfig::default(),
        ))
        .unwrap();

        let engine_score = engine.score(&layout).unwrap();

        assert_eq!(
            oracle_score, engine_score.0,
            "Oracle and Generic Engine must match exactly"
        );
    }
}
