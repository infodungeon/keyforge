// libs/keyforge-model/tests/cost_model_test.rs

use keyforge_model::cost_model::{CostModel, FingerDefinition};
use serde_json;

#[test]
fn test_cost_model_deserialization() {
    let json_data = r#"{
  "meta": {
    "version": "2.0",
    "description": "KeyForge Physics Cost Matrix",
    "unit": "Effort Points (100 = Index Home)"
  },
  "models": {
    "model_a_row_staggered": {
      "description": "Standard ANSI/ISO Geometry",
      "static_costs": {
        "left_hand": {
          "pinky": {
            "base":  { "r0": 244, "r1": 196, "r2": 145, "r3": 207 },
            "outer": { "r0": 244, "r1": 210, "r2": 154, "r3": 207 },
            "ext":   { "r0": 281, "r1": 210, "r2": 182, "r3": 215, "r4": 239 }
          },
          "thumb": {
            "pos_1": 105,
            "pos_2": 125,
            "pos_3": 145
          }
        }
      }
    }
  },
  "dynamic_rules": {
    "sequence_modifiers": {
      "roll_inward": -0.30
    },
    "penalties": {
      "sfb_multiplier": 2.5
    },
    "constraints": {
      "hand_balance_tolerance": 0.05
    }
  }
}"#;

    let model: CostModel = serde_json::from_str(json_data).expect("Failed to deserialize CostModel");

    assert_eq!(model.meta.version, "2.0");
    
    let model_a = model.models.get("model_a_row_staggered").expect("Missing model_a");
    let left_hand = model_a.static_costs.get("left_hand").expect("Missing left_hand");
    
    // Check Pinky (Standard)
    let pinky = left_hand.fingers.get("pinky").expect("Missing pinky");
    match pinky {
        FingerDefinition::Standard(zones) => {
            let base = zones.get("base").expect("Missing base zone");
            let r0_cost = base.get("r0").expect("Missing r0 cost");
            assert_eq!(*r0_cost, 244.0);
        },
        _ => panic!("Pinky should be Standard definition"),
    }

    // Check Thumb (Flat)
    let thumb = left_hand.fingers.get("thumb").expect("Missing thumb");
    match thumb {
        FingerDefinition::Thumb(positions) => {
            let pos1 = positions.get("pos_1").expect("Missing pos_1");
            assert_eq!(*pos1, 105.0);
        },
        _ => panic!("Thumb should be Thumb definition"),
    }
}
