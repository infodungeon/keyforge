use super::CompilationStage;
use crate::errors::PhysicsError;
use crate::kernel::mechanics::calculate_pair_cost;
use crate::kernel::types::{KeyIndex, Score};
use keyforge_model::cost_model::{FingerDefinition, HandDefinition};
use keyforge_model::types::{FingerIndex, HandIndex};
use keyforge_model::{CostModel, KeyNode, Keyboard, Rubric};
use tracing::warn;

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
}

impl CompilationStage for CostStage<'_> {
    type Input = ();
    type Output = CostOutput;

    fn execute(&self, (): Self::Input) -> Result<Self::Output, PhysicsError> {
        let key_count = self.kb.count();
        let model_key = "model_a_row_staggered";
        let phys_model = self
            .cost_model
            .models
            .get(model_key)
            .ok_or_else(|| PhysicsError::Config(format!("Missing cost model: {model_key}")))?;

        let mut key_costs = Vec::with_capacity(key_count);
        for k in &self.kb.keys {
            let cost_val = resolve_key_cost(k, &phys_model.static_costs);
            key_costs.push(Score::from_f32(cost_val));
        }

        let mut internal_cost_matrix = vec![Score::ZERO; key_count * key_count];
        for i in 0..key_count {
            for j in 0..key_count {
                let cost =
                    calculate_pair_cost(self.kb, self.rubric, KeyIndex::from(i), KeyIndex::from(j));
                internal_cost_matrix[i * key_count + j] = Score::from_f32(cost);
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

    warn!("Cost lookup failed for key {:?}, using default 100.0", key);
    100.0
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
            finger: FingerIndex(1),
            row: RowIndex(0),
            col: ColIndex(0),
            ..Default::default()
        };

        let cost = resolve_key_cost(&key, &static_costs);
        assert_eq!(cost, 10.0);
    }
}
