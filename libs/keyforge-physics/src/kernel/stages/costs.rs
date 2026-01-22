use super::CompilationStage;
use crate::error::PhysicsError;
use crate::kernel::mechanics::calculate_pair_cost;
use crate::kernel::types::{KeyIndex, Score};
use keyforge_model::cost_model::{FingerDefinition, HandDefinition};
use keyforge_model::types::{FingerIndex, HandIndex};
use keyforge_model::{CostModel, KeyNode, Keyboard, Rubric};

/// Intermediate state containing key costs mapped from the cost model.
pub struct CostOutput {
    pub key_costs: Vec<Score>,
    pub cost_matrix: Vec<Score>,
}

/// Stage 2: Static Costs.
pub struct CostStage<'a> {
    pub kb: &'a Keyboard,
    pub rubric: &'a Rubric,
    pub cost_model: &'a CostModel,
    pub model_key: Option<&'a str>,
}

impl CompilationStage for CostStage<'_> {
    type Input = ();
    type Output = CostOutput;

    fn execute(&self, (): Self::Input) -> Result<Self::Output, PhysicsError> {
        let key_count = self.kb.count();
        let model_key = self.model_key.unwrap_or("model_a_row_staggered");
        let phys_model = self
            .cost_model
            .models
            .get(model_key)
            .ok_or_else(|| PhysicsError::Config(format!("Missing cost model: {model_key}")))?;

        let mut key_costs = Vec::with_capacity(key_count);
        for k in &self.kb.keys {
            let static_cost = Score::from_f32(resolve_key_cost(k, &phys_model.static_costs)?)
                .map_err(|e| PhysicsError::InvalidInput { message: e })?;
            let finger_effort = Score::from_f32(self.rubric.finger_effort[k.finger.as_usize()])
                .map_err(|e| PhysicsError::InvalidInput { message: e })?;

            key_costs.push(static_cost.checked_add(finger_effort).ok_or_else(|| {
                PhysicsError::ScoreOverflow {
                    context: format!("Static key cost accumulation for key {}", k.index),
                }
            })?);
        }

        let mut internal_cost_matrix = vec![Score::ZERO; key_count * key_count];
        for i in 0..key_count {
            for j in 0..key_count {
                let cost = calculate_pair_cost(
                    self.kb,
                    self.rubric,
                    KeyIndex::from(i),
                    KeyIndex::from(j),
                )?;
                internal_cost_matrix[i * key_count + j] = Score(cost);
            }
        }

        Ok(CostOutput {
            key_costs,
            cost_matrix: internal_cost_matrix,
        })
    }
}

fn resolve_key_cost(
    key: &KeyNode,
    static_costs: &std::collections::HashMap<String, HandDefinition>,
) -> Result<f32, PhysicsError> {
    const ZONE_INNER_THRESHOLD: i8 = 1;
    const ZONE_OUTER_THRESHOLD: i8 = 1;

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
                    let zone_key = if key.col.0.unsigned_abs() > ZONE_INNER_THRESHOLD as u8
                        && key.finger == FingerIndex::INDEX
                    {
                        "inner"
                    } else if key.col.0.unsigned_abs() > ZONE_OUTER_THRESHOLD as u8
                        && key.finger == FingerIndex::PINKY
                    {
                        "outer"
                    } else {
                        "base"
                    };

                    if let Some(zone) = zones.get(zone_key).or_else(|| zones.get("base")) {
                        let row_key = format!("r{}", key.row.0);
                        return Ok(zone.get(&row_key).copied().unwrap_or(0.0));
                    }
                    return Ok(0.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::types::{ColIndex, RowIndex};

    #[test]
    fn test_resolve_key_cost_logic() {
        let mut static_costs = std::collections::HashMap::new();
        let mut hand_def = HandDefinition {
            fingers: std::collections::HashMap::new(),
        };
        let mut index_zones = std::collections::HashMap::new();
        let mut base_zone = std::collections::HashMap::new();
        base_zone.insert("r0".to_string(), 10.0);
        index_zones.insert("base".to_string(), base_zone);
        hand_def
            .fingers
            .insert("index".to_string(), FingerDefinition::Standard(index_zones));
        static_costs.insert("universal_hand".to_string(), hand_def);

        let key = KeyNode {
            index: 0,
            hand: HandIndex(0),
            finger: FingerIndex::new_unchecked(1),
            row: RowIndex(0),
            col: ColIndex(0),
            ..Default::default()
        };

        let cost = resolve_key_cost(&key, &static_costs).unwrap();
        assert_eq!(cost, 10.0);
    }

    #[test]
    fn test_resolve_key_cost_zones() {
        let mut static_costs = std::collections::HashMap::new();
        let mut fingers = std::collections::HashMap::new();

        let mut zones = std::collections::HashMap::new();
        let mut base_r0 = std::collections::HashMap::new();
        base_r0.insert("r0".to_string(), 1.0);
        zones.insert("base".to_string(), base_r0);

        let mut inner_r0 = std::collections::HashMap::new();
        inner_r0.insert("r0".to_string(), 5.0);
        zones.insert("inner".to_string(), inner_r0);

        fingers.insert("index".to_string(), FingerDefinition::Standard(zones));
        static_costs.insert("universal_hand".to_string(), HandDefinition { fingers });

        // Index finger, col 0 (base)
        let k_base = KeyNode {
            finger: FingerIndex::INDEX,
            col: ColIndex(0),
            ..Default::default()
        };
        assert_eq!(resolve_key_cost(&k_base, &static_costs).unwrap(), 1.0);

        // Index finger, col 2 (inner)
        let k_inner = KeyNode {
            finger: FingerIndex::INDEX,
            col: ColIndex(2),
            ..Default::default()
        };
        assert_eq!(resolve_key_cost(&k_inner, &static_costs).unwrap(), 5.0);

        // Index finger, col -128 (inner, via unsigned_abs)
        let k_min = KeyNode {
            finger: FingerIndex::INDEX,
            col: ColIndex(-128),
            ..Default::default()
        };
        assert_eq!(resolve_key_cost(&k_min, &static_costs).unwrap(), 5.0);
    }
}
