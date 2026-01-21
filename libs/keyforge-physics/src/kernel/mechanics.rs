// libs/keyforge-physics/src/kernel/mechanics.rs

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

use super::types::{FingerIndex, KeyIndex};
use crate::error::PhysicsError;
use keyforge_model::{Keyboard, Rubric};

fn to_score_or_err(val: f32) -> Result<i64, PhysicsError> {
    keyforge_model::types::Score::from_f32(val)
        .map(|s| s.0)
        .map_err(|e| PhysicsError::InvalidInput { message: e })
}

pub fn calculate_pair_cost(kb: &Keyboard, rubric: &Rubric, i: KeyIndex, j: KeyIndex) -> Result<i64, PhysicsError> {
    let i_idx = usize::from(i);
    let j_idx = usize::from(j);

    if i_idx == j_idx {
        return Ok(0);
    }

    let k1 = &kb.keys[i_idx];
    let k2 = &kb.keys[j_idx];

    let h1 = k1.hand;
    let h2 = k2.hand;
    let f1 = k1.finger;
    let f2 = k2.finger;

    if h1 != h2 {
        return Ok(0);
    }

    let (dx2, dy2) = kb.spatial_cache[i_idx * kb.keys.len() + j_idx];
    
    // Intermediate geometric math in f64
    let t_lat = f64::from(rubric.travel_lat);
    let t_vert = f64::from(rubric.travel_vert);
    let scale = f64::from(keyforge_model::constants::SCORE_SCALE);
    
    let dist_raw = ((dx2 as f64 * t_lat) + (dy2 as f64 * t_vert)) * scale;
    
    if dist_raw.is_nan() || dist_raw.is_infinite() {
        return Err(PhysicsError::InvalidInput { 
            message: format!("Geometric distance between keys {} and {} is invalid (NaN or Infinite)", i, j) 
        });
    }

    let mut cost = dist_raw.round() as i64;

    if f1 == f2 {
        let mut reach_k2 = 0.0f64;
        if let Some(origin) = kb
            .finger_origins
            .get(k2.hand.as_usize())
            .and_then(|h| h.get(k2.finger.as_usize()))
        {
            // Effort model: Parabolic cost for reach distance.
            // Cost scales with the square of the distance from the home position (origin).
            // `t_lat` and `t_vert` weight the horizontal and vertical components respectively.
            let odx = (k2.x - origin.0) as f64;
            let ody = (k2.y - origin.1) as f64;
            reach_k2 = ((odx * odx * t_lat) + (ody * ody * t_vert)) * scale;
        }

        cost = cost.checked_sub(reach_k2.round() as i64).ok_or_else(|| PhysicsError::ScoreOverflow {
            context: "Pair cost reach reduction".to_string()
        })?;

        let row_diff = (k1.row.0 as i32 - k2.row.0 as i32).unsigned_abs();
        let col_diff = (k1.col.0 as i32 - k2.col.0 as i32).unsigned_abs();

        if col_diff == 1 {
            let sfb_extra = if f1.is_weak() { rubric.sfb_lateral_weak } else { rubric.sfb_lateral };
            cost = cost.checked_add(to_score_or_err(sfb_extra)?)
                .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Pair cost SFB lateral".to_string() })?;
        } else if col_diff > 1 {
            cost = cost.checked_add(to_score_or_err(rubric.sfb_diagonal)?)
                .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Pair cost SFB diagonal".to_string() })?;
        } else if row_diff >= u32::from(rubric.threshold_sfb_long_row_diff as u8) {
            cost = cost.checked_add(to_score_or_err(rubric.sfb_long)?)
                .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Pair cost SFB long".to_string() })?;
        } else {
            cost = cost.checked_add(to_score_or_err(rubric.sfb_base)?)
                .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Pair cost SFB base".to_string() })?;
        }
        return Ok(cost);
    }

    let finger_diff = f1.distance(f2);
    let row_diff = (k1.row.0 as i32 - k2.row.0 as i32).unsigned_abs();

    if finger_diff == 1 && f1 != FingerIndex::THUMB && f2 != FingerIndex::THUMB {
        if row_diff >= u32::from(rubric.threshold_scissor_row_diff as u8) {
            cost = cost.checked_add(to_score_or_err(rubric.penalty_scissor)?)
                .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Pair cost scissor".to_string() })?;
        } else if row_diff == 0 {
            let col_diff = (k1.col.0 as i32 - k2.col.0 as i32).unsigned_abs();
            if col_diff > 1 {
                cost = cost.checked_add(to_score_or_err(rubric.sfb_lateral)?)
                    .ok_or_else(|| PhysicsError::ScoreOverflow { context: "Pair cost lateral SFB adjacent".to_string() })?;
            }
        }
    }

    Ok(cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};
    use keyforge_model::KeyNode;

    fn setup_kb_pair() -> Keyboard {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(0),
                col: ColIndex(0),
                x: 0.0,
                y: 0.0,
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                row: RowIndex(1),
                col: ColIndex(0),
                x: 0.0,
                y: 1.0,
                is_home: false,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(2),
                row: RowIndex(1),
                col: ColIndex(1),
                x: 1.0,
                y: 1.0,
                is_home: false,
                ..Default::default()
            },
        ];
        Keyboard::new(keys, 0, "test".into()).unwrap()
    }

    #[test]
    fn test_calculate_pair_cost_sfb() {
        let kb = setup_kb_pair();
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1)).unwrap();
        assert!(cost >= crate::verify::to_fixed(rubric.sfb_base), "SFB should be penalized");
    }

    #[test]
    fn test_calculate_pair_cost_different_hands() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex(1),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, 0, "test".into()).unwrap();
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1)).unwrap();
        assert_eq!(cost, 0, "Different hands should have 0 cost");
    }

    #[test]
    fn test_calculate_pair_cost_invalid_math() {
        let kb = setup_kb_pair();
        let mut rubric = Rubric::default();
        
        // Force NaN via infinity * 0 or similar if possible, or just inject Infinity
        rubric.travel_lat = f32::INFINITY;
        let res = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1));
        assert!(matches!(res, Err(PhysicsError::InvalidInput { .. })));

        rubric.travel_lat = f32::NAN;
        let res = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1));
        assert!(matches!(res, Err(PhysicsError::InvalidInput { .. })));
    }

    #[test]
    fn test_calculate_pair_cost_overflows() {
        let kb = setup_kb_pair();
        let mut rubric = Rubric::default();
        
        // 1. checked_add overflow (SFB base)
        rubric.sfb_base = 1e20; // Massive value
        let res = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1));
        assert!(matches!(res, Err(PhysicsError::InvalidInput { .. }))); // Score::from_f32 fails first
        
        // To hit ScoreOverflow we need Score::from_f32 to succeed but the i64 add to fail.
        // Score::from_f32 checks bounds against Score::MAX.
        // So we need cost + penalty > i64::MAX.
        // If we set sfb_base to Score::MAX / 2 and dist_raw to Score::MAX / 2.
        
        // Actually, Score::from_f32(1e20) returns Err(String).
        // Mechanics maps this to PhysicsError::InvalidInput.
    }
}
