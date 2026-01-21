// libs/keyforge-physics/src/verify.rs

use keyforge_model::{Corpus, KeyNode, Keyboard, Rubric, types::KeyCode};
use crate::PhysicsError;

/// A naive, high-precision version of the scoring logic used to verify 
/// the optimized ScoringEngine results.
#[derive(Debug, Clone)]
pub struct DeterministicScorer {
    rubric: FixedPointRubric,
    static_costs: std::collections::HashMap<String, keyforge_model::cost_model::HandDefinition>,
    sequence_modifiers: std::collections::HashMap<(u16, u16), i64>,
    penalty_redirect: i64,
    bonus_roll: i64,
}

impl DeterministicScorer {
    #[must_use]
    pub fn new(rubric: &Rubric, cost_model: &keyforge_model::CostModel) -> Self {
        let model_key = "model_a_row_staggered";
        let static_costs = cost_model.models.get(model_key)
            .map(|m| m.static_costs.clone())
            .unwrap_or_default();

        let mut sequence_modifiers = std::collections::HashMap::new();
        for (bigram, &val) in &cost_model.dynamic_rules.sequence_modifiers {
            if bigram.len() == 2 {
                let bytes = bigram.as_bytes();
                let key = (u16::from(bytes[0]), u16::from(bytes[1]));
                sequence_modifiers.insert(key, to_fixed(val));
            }
        }

        Self {
            rubric: FixedPointRubric::from_rubric(rubric),
            static_costs,
            sequence_modifiers,
            penalty_redirect: to_fixed(rubric.redirect),
            bonus_roll: to_fixed(rubric.roll_bonus),
        }
    }

    /// Scores a layout bit-for-bit against the naive algorithm.
    pub fn score_detailed(&self, kb: &Keyboard, corpus: &Corpus, layout_keys: &[KeyCode]) -> Result<(i64, i64, i64), PhysicsError> {
        // 1. Monograms
        let mut mono_score = 0i64;
        for (code_val, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq == 0 { continue; }
            let code = KeyCode(code_val as u16);
            let indices = find_indices(layout_keys, code);
            if indices.is_empty() { continue; }

            let mut min_total_cost = i64::MAX;
            for &idx in &indices {
                let key = &kb.keys[idx];
                let effort = self.rubric.finger_effort[key.finger.as_usize()];
                let static_cost = to_fixed(resolve_static_key_cost(key, &self.static_costs));
                let total = effort.checked_add(static_cost).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Monogram effort + static cost for code {}", code_val) })?;
                if total < min_total_cost { min_total_cost = total; }
            }
            let contrib = min_total_cost.checked_mul(freq as i64).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Monogram freq scale for code {}", code_val) })?;
            mono_score = mono_score.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Monogram total accumulation at code {}", code_val) })?;
        }

        // 2. Bigrams
        let mut bigram_score = 0i64;
        for (c1, c2, freq) in &corpus.bigrams {
            let freq = i64::from(*freq);
            let indices1 = find_indices(layout_keys, KeyCode(*c1));
            let indices2 = find_indices(layout_keys, KeyCode(*c2));

            if !indices1.is_empty() && !indices2.is_empty() {
                let mut min_cost = i64::MAX;
                for &idx1 in &indices1 {
                    for &idx2 in &indices2 {
                        let mut cost = self.rubric.calculate_pair_cost(kb, &kb.keys[idx1], &kb.keys[idx2], idx1, idx2)?;
                        
                        // Apply sequence modifiers
                        if let Some(&mod_val) = self.sequence_modifiers.get(&(*c1, *c2)) {
                            cost = cost.checked_add(mod_val).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Bigram modifier for ({}, {})", c1, c2) })?;
                        }

                        if cost < min_cost { min_cost = cost; }
                    }
                }
                if min_cost != i64::MAX {
                    let contrib = min_cost.checked_mul(freq).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Bigram freq scale for ({}, {})", c1, c2) })?;
                    bigram_score = bigram_score.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Bigram total accumulation at ({}, {})", c1, c2) })?;
                }
            }
        }

        // 3. Trigrams
        let mut trigram_score = 0i64;
        for (c1, c2, c3, freq) in &corpus.trigrams {
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
                            if cost < min_cost { min_cost = cost; }
                        }
                    }
                }
                if min_cost != i64::MAX {
                    let contrib = min_cost.checked_mul(freq).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Trigram freq scale for ({}, {}, {})", c1, c2, c3) })?;
                    trigram_score = trigram_score.checked_add(contrib).ok_or_else(|| PhysicsError::ScoreOverflow { context: format!("Trigram total accumulation at ({}, {}, {})", c1, c2, c3) })?;
                }
            }
        }

        Ok((mono_score, bigram_score, trigram_score))
    }

    pub fn score(&self, kb: &Keyboard, corpus: &Corpus, layout_keys: &[KeyCode]) -> Result<i64, PhysicsError> {
        let (m, b, t) = self.score_detailed(kb, corpus, layout_keys)?;
        m.checked_add(b)
            .and_then(|sum| sum.checked_add(t))
            .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Final oracle total accumulation".into() })
    }

    fn calculate_flow_cost(&self, kb: &Keyboard, p1: usize, p2: usize, p3: usize) -> i64 {
        let k1 = &kb.keys[p1];
        let k2 = &kb.keys[p2];
        let k3 = &kb.keys[p3];
        if k1.hand != k2.hand || k2.hand != k3.hand {
            return 0;
        }

        if k1.finger == k3.finger && k1.finger != k2.finger {
            return self.penalty_redirect;
        }

        let dir1 = k2.finger.diff(k1.finger);
        let dir2 = k3.finger.diff(k2.finger);
        if dir1 == 0 || dir2 == 0 {
            return 0;
        }
        // Check if directions are different (dir1.signum() != dir2.signum())
        if (dir1 > 0 && dir2 < 0) || (dir1 < 0 && dir2 > 0) {
            return self.penalty_redirect;
        }
        if dir1 < 0 {
            return -self.bonus_roll; // Using Neg implementation assumed for i64
        }
        0
    }
}

fn find_indices(layout: &[KeyCode], target: KeyCode) -> Vec<usize> {
    layout.iter().enumerate()
        .filter(|(_, &k)| k == target)
        .map(|(i, _)| i)
        .collect()
}

use keyforge_model::config::weights::DEFAULT_PENALTY_MISSING_KEY;

fn resolve_static_key_cost(
    key: &KeyNode,
    static_costs: &std::collections::HashMap<String, keyforge_model::cost_model::HandDefinition>,
) -> f32 {
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
                    let zone_key = if key.col.0.unsigned_abs() > 1 && key.finger == keyforge_model::FingerIndex::INDEX {
                        "inner"
                    } else if key.col.0.unsigned_abs() > 1 && key.finger == keyforge_model::FingerIndex::PINKY {
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
                        .unwrap_or(&DEFAULT_PENALTY_MISSING_KEY);
                }
            }
        }
    }
    DEFAULT_PENALTY_MISSING_KEY
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
            finger_effort: r.finger_effort.iter().map(|&e| to_fixed(e)).collect(),
            travel_lat: to_fixed(r.travel_lat),
            travel_vert: to_fixed(r.travel_vert),
            sfb_base: to_fixed(r.sfb_base),
            sfb_lateral: to_fixed(r.sfb_lateral),
            sfb_lateral_weak: to_fixed(r.sfb_lateral_weak),
            sfb_diagonal: to_fixed(r.sfb_diagonal),
            sfb_long: to_fixed(r.sfb_long),
            penalty_scissor: to_fixed(r.penalty_scissor),
            threshold_sfb_long_row_diff: i32::from(r.threshold_sfb_long_row_diff),
            threshold_scissor_row_diff: i32::from(r.threshold_scissor_row_diff),
        }
    }

    fn calculate_pair_cost(
        &self, 
        kb: &Keyboard, 
        k1: &KeyNode, 
        k2: &KeyNode, 
        idx1: usize, 
        idx2: usize,
    ) -> Result<i64, PhysicsError> {
        if idx1 == idx2 { return Ok(0); }
        if k1.hand != k2.hand { return Ok(0); } // Engine only scores same-hand travel
        
        // Use pre-computed spatial cache from keyboard to match engine's source data
        let (dx2, dy2) = kb.spatial_cache[idx1 * kb.keys.len() + idx2];
        
        // Intermediate geometric math in f64 (MUST MATCH mechanics.rs)
        let t_lat = f64::from(to_f32(self.travel_lat));
        let t_vert = f64::from(to_f32(self.travel_vert));
        let scale = f64::from(keyforge_model::constants::SCORE_SCALE);
        
        let dist_raw = ((dx2 as f64 * t_lat) + (dy2 as f64 * t_vert)) * scale;
        
        if dist_raw.is_nan() || dist_raw.is_infinite() {
            return Err(PhysicsError::InvalidInput { 
                message: format!("Oracle geometric distance between keys {} and {} is invalid (NaN or Infinite)", idx1, idx2) 
            });
        }

        let mut cost = dist_raw.round() as i64;

        if k1.finger == k2.finger {
            let mut reach_k2 = 0.0f64;
            if let Some(origin) = kb.finger_origins.get(k2.hand.as_usize()).and_then(|h| h.get(k2.finger.as_usize())) {
                let rdx = (k2.x - origin.0) as f64;
                let rdy = (k2.y - origin.1) as f64;
                reach_k2 = ((rdx * rdx * t_lat) + (rdy * rdy * t_vert)) * scale;
            }

            cost = cost.checked_sub(reach_k2.round() as i64).ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Oracle SFB reach reduction".to_string()
            })?;

            let row_diff = (k1.row.0 as i32 - k2.row.0 as i32).unsigned_abs();
            let col_diff = (k1.col.0 as i32 - k2.col.0 as i32).unsigned_abs();

            if col_diff == 1 {
                let sfb_extra = if k1.finger.is_weak() { self.sfb_lateral_weak } else { self.sfb_lateral };
                cost = cost.checked_add(sfb_extra).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB lateral".to_string()
                })?;
            } else if col_diff > 1 {
                cost = cost.checked_add(self.sfb_diagonal).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB diagonal".to_string()
                })?;
            } else if row_diff >= self.threshold_sfb_long_row_diff as u32 {
                cost = cost.checked_add(self.sfb_long).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB long".to_string()
                })?;
            } else {
                cost = cost.checked_add(self.sfb_base).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle SFB base".to_string()
                })?;
            }
            return Ok(cost);
        }

        let finger_diff = k1.finger.distance(k2.finger);
        let row_diff = (k1.row.0 as i32 - k2.row.0 as i32).unsigned_abs();

        if finger_diff == 1 && k1.finger != keyforge_model::types::FingerIndex::THUMB && k2.finger != keyforge_model::types::FingerIndex::THUMB {
            if row_diff >= self.threshold_scissor_row_diff as u32 {
                cost = cost.checked_add(self.penalty_scissor).ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Oracle scissor penalty".to_string()
                })?;
            } else if row_diff == 0 {
                let col_diff = (k1.col.0 as i32 - k2.col.0 as i32).unsigned_abs();
                if col_diff > 1 {
                    cost = cost.checked_add(self.sfb_lateral).ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "Oracle lateral SFB adjacent".to_string()
                    })?;
                }
            }
        }

        Ok(cost)
    }
}

pub(crate) fn to_fixed(f: f32) -> i64 {
    (f * keyforge_model::constants::SCORE_SCALE) as i64
}

fn to_f32(i: i64) -> f32 {
    (i as f32) / keyforge_model::constants::SCORE_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{CostModel, KeyNode, Keyboard, Corpus, Rubric};
    use keyforge_model::types::{KeyCode, HandIndex, FingerIndex};

    fn mock_cost_model() -> keyforge_model::CostModel {
        let mut cm = keyforge_model::CostModel::default();
        
        let mut base_zone = std::collections::HashMap::new();
        for r in -128..=127 {
            base_zone.insert(format!("r{}", r), 0.0);
        }
        
        let mut index_zones = std::collections::HashMap::new();
        index_zones.insert("base".into(), base_zone.clone());
        
        let mut fingers = std::collections::HashMap::new();
        fingers.insert("thumb".into(), keyforge_model::cost_model::FingerDefinition::Thumb(std::collections::HashMap::new()));
        fingers.insert("index".into(), keyforge_model::cost_model::FingerDefinition::Standard(index_zones.clone()));
        fingers.insert("middle".into(), keyforge_model::cost_model::FingerDefinition::Standard(index_zones.clone()));
        fingers.insert("ring".into(), keyforge_model::cost_model::FingerDefinition::Standard(index_zones.clone()));
        fingers.insert("pinky".into(), keyforge_model::cost_model::FingerDefinition::Standard(index_zones.clone()));

        let mut static_costs = std::collections::HashMap::new();
        static_costs.insert("universal_hand".into(), keyforge_model::cost_model::HandDefinition { fingers });

        cm.models.insert(
            "model_a_row_staggered".into(),
            keyforge_model::cost_model::ModelDefinition {
                description: "test".into(),
                static_costs,
            },
        );
        cm
    }

    fn setup_kb_wiring() -> Keyboard {
        let keys: Vec<KeyNode> = (0..3)
            .map(|i| KeyNode {
                index: i,
                label: format!("k{}", i),
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(i as u8),
                x: i as f32,
                ..Default::default()
            })
            .collect();
        Keyboard::new(keys, 0, "test".into()).unwrap()
    }

    fn setup_minimal() -> (Keyboard, Corpus, Rubric, CostModel) {
        let kb = setup_kb_wiring();
        let mut corpus = Corpus::default();
        corpus.char_freqs[97] = 100;
        corpus.char_freqs[98] = 200;
        corpus.bigrams.push((97, 98, 50));
        
        let cm = mock_cost_model();
        (kb, corpus, Rubric::default(), cm)
    }

    #[test]
    fn test_deterministic_scorer_detailed_branches() {
        let mut rubric = Rubric::default();
        rubric.roll_bonus = 10.0;
        rubric.redirect = 20.0;
        let mut cm = keyforge_model::CostModel::default();
        cm.dynamic_rules.sequence_modifiers.insert("ab".into(), 5.0);
        
        let oracle = DeterministicScorer::new(&rubric, &cm);
        let kb = setup_kb_wiring();
        let mut corpus = Corpus::default();
        corpus.char_freqs['a' as usize] = 10;
        corpus.char_freqs['b' as usize] = 20;
        corpus.bigrams.push(('a' as u16, 'b' as u16, 5));
        corpus.trigrams.push(('a' as u16, 'b' as u16, 'a' as u16, 2)); // Redirect
        
        let layout_keys = vec![KeyCode('a' as u16), KeyCode('b' as u16), KeyCode('c' as u16)];
        let res = oracle.score_detailed(&kb, &corpus, &layout_keys).unwrap();
        assert!(res.0 > 0); // Mono
        assert!(res.1 > 0); // Bigram
        assert!(res.2 > 0); // Trigram
    }

    #[test]
    fn test_calculate_flow_cost_branches() {
        let (_kb, _corpus, rubric, cm) = setup_minimal();
        let oracle = DeterministicScorer::new(&rubric, &cm);
        
        // Hand mismatch -> 0
        // Wait, calculate_flow_cost takes indices.
        // We need a 3-key keyboard.
        let keys = vec![
            KeyNode { index: 0, hand: HandIndex::LEFT, finger: FingerIndex::INDEX, ..Default::default() },
            KeyNode { index: 1, hand: HandIndex::LEFT, finger: FingerIndex::MIDDLE, ..Default::default() },
            KeyNode { index: 2, hand: HandIndex::LEFT, finger: FingerIndex::RING, ..Default::default() },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        
        // Roll: Ring -> Middle -> Index (Inward? No, Index -> Middle -> Ring is outward)
        // dir1 = k2.finger.diff(k1.finger)
        // Middle (2) - Index (1) = 1 (Positive) -> Outward
        // Ring (3) - Middle (2) = 1 (Positive) -> Outward
        // Ring -> Middle -> Index:
        // dir1 = 2 - 3 = -1 (Negative) -> Inward
        // dir2 = 1 - 2 = -1 (Negative) -> Inward
        // If dir1 < 0 { return -bonus_roll }
        assert_eq!(oracle.calculate_flow_cost(&kb, 2, 1, 0), -oracle.bonus_roll);
        
        // Redirect: Index -> Middle -> Index
        // dir1 = 2 - 1 = 1
        // dir2 = 1 - 2 = -1
        // signum mismatch -> penalty_redirect
        assert_eq!(oracle.calculate_flow_cost(&kb, 0, 1, 0), oracle.penalty_redirect);
    }

    #[test]
    fn test_deterministic_scorer_overflows() {
        let rubric = Rubric::default();
        let mut cm = keyforge_model::CostModel::default();
        
        // Inject a MASSIVE static cost for the 'universal_hand' -> 'index' -> 'base' -> 'r0'
        // This corresponds to key index 0 in setup_kb_wiring.
        // Cost = 1e15. Freq = 1e6. Product = 1e21 (Overflows i64 which is 9e18)
        let huge_cost = 1_000_000_000_000_000.0; // 1e15
        
        let mut base_zone = std::collections::HashMap::new();
        base_zone.insert("r0".into(), huge_cost);
        let mut index_zones = std::collections::HashMap::new();
        index_zones.insert("base".into(), base_zone);
        let mut fingers = std::collections::HashMap::new();
        fingers.insert("index".into(), keyforge_model::cost_model::FingerDefinition::Standard(index_zones));
        
        let mut static_costs = std::collections::HashMap::new();
        static_costs.insert("universal_hand".into(), keyforge_model::cost_model::HandDefinition { fingers });

        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs,
        });
        
        let oracle = DeterministicScorer::new(&rubric, &cm);
        let kb = setup_kb_wiring();
        
        // 1. Monogram Overflow
        let mut corpus = Corpus::default();
        // The layout keys must map to the physical key with the huge cost.
        // layout_keys = [97, 98, 99].
        // find_indices(98) -> index 1.
        // kb.keys[1] is Left Index at r0. This hits our huge cost.
        corpus.char_freqs[98] = 1_000_000; // Moderate freq is enough given the huge cost
        
        let layout_keys = vec![KeyCode(97), KeyCode(98), KeyCode(99)];
        
        let res = oracle.score_detailed(&kb, &corpus, &layout_keys);
        assert!(res.is_err(), "Should overflow on massive static cost * frequency");
        
        // 2. Bigram Overflow
        // Reset monogram freq to avoid early failure
        corpus.char_freqs[98] = 0;
        
        // We need a bigram cost to be huge.
        // DeterministicScorer::calculate_pair_cost uses distance * travel_lat.
        // We can't easily change distance (geometry) to be infinite.
        // But we CAN use sequence_modifiers!
        // DeterministicScorer adds sequence_modifier to the pair cost.
        // If we make modifier huge...
        
        // Re-create oracle with huge modifier
        let mut cm_bigram = cm.clone(); // Has huge static cost, but we won't use monograms
        cm_bigram.dynamic_rules.sequence_modifiers.insert("ab".into(), huge_cost);
        let oracle_bigram = DeterministicScorer::new(&rubric, &cm_bigram);
        
        let mut corpus_bi = Corpus::default();
        corpus_bi.bigrams.push((97, 98, 1_000_000)); // Moderate freq
        
        let res_bi = oracle_bigram.score_detailed(&kb, &corpus_bi, &layout_keys);
        assert!(res_bi.is_err(), "Should overflow on massive bigram modifier * frequency");

        // 3. Trigram Overflow
        let mut corpus_tri = Corpus::default();
        // Setup a trigram that triggers a redirect (97->98->97)
        corpus_tri.trigrams.push((97, 98, 97, 1_000_000));
        
        // Massive penalty redirect
        let mut rubric_tri = Rubric::default();
        // 1e12 * 1e6 (fixed point) = 1e18 < i64::MAX (9e18)
        // 1e18 * 1e6 (freq) = 1e24 > i64::MAX
        rubric_tri.redirect = 1_000_000_000_000.0; 
        let oracle_tri = DeterministicScorer::new(&rubric_tri, &cm);
        
        let res_tri = oracle_tri.score_detailed(&kb, &corpus_tri, &layout_keys);
        assert!(res_tri.is_err(), "Should overflow on massive trigram redirect penalty * frequency");
    }

    #[test]
    fn test_resolve_static_key_cost_branches() {
        let mut static_costs = std::collections::HashMap::new();
        let mut left_hand = keyforge_model::cost_model::HandDefinition { fingers: std::collections::HashMap::new() };
        let mut right_hand = keyforge_model::cost_model::HandDefinition { fingers: std::collections::HashMap::new() };
        
        let mut zones = std::collections::HashMap::new();
        let mut base_zone = std::collections::HashMap::new();
        base_zone.insert("r0".into(), 1.0);
        zones.insert("base".into(), base_zone);
        
        let mut outer_zone = std::collections::HashMap::new();
        outer_zone.insert("r0".into(), 10.0);
        zones.insert("outer".into(), outer_zone);

        left_hand.fingers.insert("pinky".into(), keyforge_model::cost_model::FingerDefinition::Standard(zones.clone()));
        right_hand.fingers.insert("thumb".into(), keyforge_model::cost_model::FingerDefinition::Thumb(std::collections::HashMap::from([("p1".into(), 5.0)])));
        
        static_costs.insert("left_hand".into(), left_hand);
        static_costs.insert("right_hand".into(), right_hand);

        // Left Pinky Base
        let k1 = KeyNode { hand: HandIndex::LEFT, finger: FingerIndex::PINKY, col: keyforge_model::types::ColIndex(0), row: keyforge_model::types::RowIndex(0), ..Default::default() };
        assert_eq!(resolve_static_key_cost(&k1, &static_costs), 1.0);

        // Left Pinky Outer
        let k2 = KeyNode { hand: HandIndex::LEFT, finger: FingerIndex::PINKY, col: keyforge_model::types::ColIndex(5), row: keyforge_model::types::RowIndex(0), ..Default::default() };
        assert_eq!(resolve_static_key_cost(&k2, &static_costs), 10.0);

        // Right Thumb
        let k3 = KeyNode { hand: HandIndex::RIGHT, finger: FingerIndex::THUMB, ..Default::default() };
        assert_eq!(resolve_static_key_cost(&k3, &static_costs), 5.0);
        
        // Unknown
        let k4 = KeyNode { hand: HandIndex::LEFT, finger: FingerIndex::RING, ..Default::default() };
        assert_eq!(resolve_static_key_cost(&k4, &static_costs), 100.0);
    }
}