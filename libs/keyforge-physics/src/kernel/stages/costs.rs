use super::CompilationStage;
use crate::error::PhysicsError;
use crate::kernel::mechanics::calculate_pair_cost;
use crate::kernel::types::{KeyIndex, Score};
use keyforge_model::cost_model::{FingerDefinition, HandDefinition};
use keyforge_model::types::{FingerIndex, HandIndex};
use keyforge_model::{CostModel, KeyNode, Keyboard, Rubric};

/// Intermediate state containing key costs mapped from the cost model.
#[derive(Debug)]
pub(crate) struct CostOutput {
    pub key_costs: Vec<Score>,
    pub cost_matrix: Vec<Score>,
}

/// Stage 2: Static Costs.
#[derive(Debug)]
pub(crate) struct CostStage<'a> {
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
            .models()
            .get(model_key)
            .ok_or_else(|| PhysicsError::Config(format!("Missing cost model: {model_key}")))?;

        let mut key_costs = Vec::with_capacity(key_count);
        for k in self.kb.keys() {
            let static_cost = resolve_key_cost(k, &phys_model.static_costs)?;
            let finger_effort = self.rubric.finger_effort()[k.finger.as_usize()];

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
                internal_cost_matrix[i * key_count + j] = Score::from_scaled_i64(cost);
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
) -> Result<Score, PhysicsError> {
    let hand = get_hand_def(key, static_costs)?;
    let finger_def = get_finger_def(key, hand)?;

    let val = match finger_def {
        FingerDefinition::Standard(reach) => resolve_standard_finger(key, reach),
        FingerDefinition::Thumb(positions) => {
            positions.values().min().copied().unwrap_or(Score::ZERO)
        }
        FingerDefinition::Fallback(_) => Score::ZERO,
    };

    Ok(val)
}

fn get_hand_def<'a>(
    key: &KeyNode,
    static_costs: &'a std::collections::HashMap<String, HandDefinition>,
) -> Result<&'a HandDefinition, PhysicsError> {
    let hand_key = if key.hand == HandIndex::LEFT {
        "left_hand"
    } else {
        "right_hand"
    };

    static_costs
        .get(hand_key)
        .or_else(|| static_costs.get("universal_hand"))
        .ok_or_else(|| {
            PhysicsError::Config(format!(
                "Hand definition not found for {hand_key} or universal_hand"
            ))
        })
}

fn get_finger_def<'a>(
    key: &KeyNode,
    hand: &'a HandDefinition,
) -> Result<&'a FingerDefinition, PhysicsError> {
    let finger_key = match key.finger {
        FingerIndex::THUMB => "thumb",
        FingerIndex::INDEX => "index",
        FingerIndex::MIDDLE => "middle",
        FingerIndex::RING => "ring",
        FingerIndex::PINKY => "pinky",
        _ => "unknown",
    };

    hand.fingers.get(finger_key).ok_or_else(|| {
        PhysicsError::Config(format!(
            "Finger {:?} ({}) not found in hand definition",
            key.finger, finger_key
        ))
    })
}

fn resolve_standard_finger(
    key: &KeyNode,
    reach: &keyforge_model::cost_model::FingerReach,
) -> Score {
    const ZONE_INNER_THRESHOLD: u8 = 1;
    const ZONE_OUTER_THRESHOLD: u8 = 1;

    let col_abs = key.col.raw().unsigned_abs();
    let zone = match key.finger {
        FingerIndex::INDEX if col_abs > ZONE_INNER_THRESHOLD => &reach.inner,
        FingerIndex::PINKY if col_abs > ZONE_OUTER_THRESHOLD => &reach.outer,
        _ => &reach.base,
    };

    // Fallback to base if specifically requested zone is empty
    let target_zone = if zone.is_empty() { &reach.base } else { zone };
    target_zone.get(&key.row).copied().unwrap_or(Score::ZERO)
}

#[keyforge_testing_macros::kf_test]
#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::types::{ColIndex, KeyIndex, RowIndex};
    use std::collections::HashMap;

    #[test]
    fn test_resolve_key_cost_logic() -> anyhow::Result<()> {
        let mut static_costs = std::collections::HashMap::new();
        let mut hand_def = HandDefinition {
            fingers: std::collections::HashMap::new(),
        };
        let mut base_zone = keyforge_model::cost_model::RowCosts::new();
        let sc = |v: i64| Score::from_scaled_i64(v);
        base_zone.insert(RowIndex::new(0), sc(10_000_000));

        let zones = keyforge_model::cost_model::FingerReach {
            base: base_zone,
            inner: HashMap::default(),
            outer: HashMap::default(),
        };

        hand_def
            .fingers
            .insert("index".to_string(), FingerDefinition::Standard(zones));
        static_costs.insert("universal_hand".to_string(), hand_def);

        let key = KeyNode {
            index: KeyIndex::new(0),
            hand: HandIndex::new(0),
            finger: FingerIndex::new_unchecked(1),
            row: RowIndex::new(0),
            col: ColIndex::new(0),
            ..Default::default()
        };

        let cost = resolve_key_cost(&key, &static_costs)?;
        assert_eq!(cost.raw(), 10_000_000);
        Ok(())
    }

    #[test]
    fn test_resolve_key_cost_zones() -> anyhow::Result<()> {
        let mut static_costs = std::collections::HashMap::new();
        let mut fingers = std::collections::HashMap::new();
        let sc = |v: i64| Score::from_scaled_i64(v);

        let mut base_r0 = keyforge_model::cost_model::RowCosts::new();
        base_r0.insert(RowIndex::new(0), sc(1_000_000));

        let mut inner_r0 = keyforge_model::cost_model::RowCosts::new();
        inner_r0.insert(RowIndex::new(0), sc(5_000_000));

        let zones = keyforge_model::cost_model::FingerReach {
            base: base_r0,
            inner: inner_r0,
            outer: HashMap::default(),
        };

        fingers.insert("index".to_string(), FingerDefinition::Standard(zones));
        static_costs.insert("universal_hand".to_string(), HandDefinition { fingers });

        // Index finger, col 0 (base)
        let k_base = KeyNode {
            finger: FingerIndex::INDEX,
            col: ColIndex::new(0),
            ..Default::default()
        };
        assert_eq!(resolve_key_cost(&k_base, &static_costs)?.raw(), 1_000_000);

        // Index finger, col 2 (inner)
        let k_inner = KeyNode {
            finger: FingerIndex::INDEX,
            col: ColIndex::new(2),
            ..Default::default()
        };
        assert_eq!(resolve_key_cost(&k_inner, &static_costs)?.raw(), 5_000_000);

        // Index finger, col -128 (inner, via unsigned_abs)
        let k_min = KeyNode {
            finger: FingerIndex::INDEX,
            col: ColIndex::new(-128),
            ..Default::default()
        };
        assert_eq!(resolve_key_cost(&k_min, &static_costs)?.raw(), 5_000_000);
        Ok(())
    }
}
