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
use keyforge_model::types::{HandIndex, Score};
use keyforge_model::{Keyboard, Rubric};

/// Calculates the flow cost (rolls, redirects) for a sequence of three keys.
/// This is a shared ground-truth implementation used by all scoring tiers.
#[inline]
#[must_use]
pub fn calculate_flow_cost(
    h1: HandIndex,
    h2: HandIndex,
    h3: HandIndex,
    f1: FingerIndex,
    f2: FingerIndex,
    f3: FingerIndex,
    penalty_redirect: Score,
    bonus_roll: Score,
    bonus_roll_out: Score,
) -> Score {
    if h1 != h2 || h2 != h3 {
        return Score::ZERO;
    }

    if f1 == f3 && f1 != f2 {
        return penalty_redirect;
    }

    let dir1 = f2.diff(f1);
    let dir2 = f3.diff(f2);
    if dir1 == 0 || dir2 == 0 {
        return Score::ZERO;
    }

    // Direction change detection (dir1.signum() != dir2.signum())
    if (dir1 > 0 && dir2 < 0) || (dir1 < 0 && dir2 > 0) {
        return penalty_redirect;
    }

    match dir1.cmp(&0) {
        std::cmp::Ordering::Less => {
            // Inward Roll (Outer -> Inner)
            // Score is negative (bonus)
            Score::ZERO.checked_sub(bonus_roll).unwrap_or(Score::MIN)
        }
        std::cmp::Ordering::Greater => {
            // Outward Roll (Inner -> Outer)
            Score::ZERO
                .checked_sub(bonus_roll_out)
                .unwrap_or(Score::MIN)
        }
        std::cmp::Ordering::Equal => Score::ZERO,
    }
}

fn to_score_or_err(val: f32) -> Result<i64, PhysicsError> {
    keyforge_model::types::Score::from_f32(val)
        .map(|s| s.raw())
        .map_err(|e| PhysicsError::InvalidInput { message: e })
}

/// Calculates the cost for a pair of keys (bigram or jump), handling geometry and SFBs.
///
/// # Errors
/// Returns `PhysicsError` if:
/// - Geometric calculations result in NaN or Infinite values.
/// - Score accumulation overflows.
pub fn calculate_pair_cost(
    kb: &Keyboard,
    rubric: &Rubric,
    i: KeyIndex,
    j: KeyIndex,
) -> Result<i64, PhysicsError> {
    let i_idx = usize::from(i);
    let j_idx = usize::from(j);

    if i_idx == j_idx {
        return Ok(0);
    }

    let k1 = &kb.keys[i_idx];
    let k2 = &kb.keys[j_idx];

    if k1.hand != k2.hand {
        return Ok(0);
    }

    let (dx2, dy2) = kb.spatial_cache[i_idx * kb.keys.len() + j_idx];

    // Intermediate geometric math in f64
    let t_lat = f64::from(rubric.travel_lat());
    let t_vert = f64::from(rubric.travel_vert());
    let scale = f64::from(keyforge_model::constants::SCORE_SCALE);

    let dist_raw = ((f64::from(dx2) * t_lat) + (f64::from(dy2) * t_vert)) * scale;

    if dist_raw.is_nan() || dist_raw.is_infinite() {
        return Err(PhysicsError::InvalidInput {
            message: format!(
                "Geometric distance between keys {i} and {j} is invalid (NaN or Infinite)"
            ),
        });
    }

    #[allow(clippy::cast_possible_truncation)]
    let mut cost = dist_raw.round() as i64;

    if k1.finger == k2.finger {
        cost = calculate_sfb_cost(kb, rubric, k1, k2, cost, scale, t_lat, t_vert)?;
    } else {
        cost = calculate_non_sfb_penalties(rubric, k1, k2, cost)?;
    }

    Ok(cost)
}

#[allow(clippy::too_many_arguments)]
fn calculate_sfb_cost(
    kb: &Keyboard,
    rubric: &Rubric,
    k1: &keyforge_model::KeyNode,
    k2: &keyforge_model::KeyNode,
    mut cost: i64,
    scale: f64,
    t_lat: f64,
    t_vert: f64,
) -> Result<i64, PhysicsError> {
    let mut reach_k2 = 0.0f64;
    if let Some(origin) = kb
        .finger_origins
        .get(k2.hand.as_usize())
        .and_then(|h| h.get(k2.finger.as_usize()))
    {
        let odx = f64::from(k2.x - origin.0);
        let ody = f64::from(k2.y - origin.1);
        reach_k2 = ((odx * odx * t_lat) + (ody * ody * t_vert)) * scale;
    }

    #[allow(clippy::cast_possible_truncation)]
    let reach_k2_rounded = reach_k2.round() as i64;
    cost = cost
        .checked_sub(reach_k2_rounded)
        .ok_or_else(|| PhysicsError::ScoreOverflow {
            context: "Pair cost reach reduction".to_string(),
        })?;

    let row_diff = (i32::from(k1.row.raw()) - i32::from(k2.row.raw())).unsigned_abs();
    let col_diff = (i32::from(k1.col.raw()) - i32::from(k2.col.raw())).unsigned_abs();

    if col_diff == 1 {
        let sfb_extra = if k1.finger.is_weak() {
            rubric.sfb_lateral_weak()
        } else {
            rubric.sfb_lateral()
        };
        cost = cost
            .checked_add(to_score_or_err(sfb_extra)?)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost SFB lateral".to_string(),
            })?;
    } else if col_diff > 1 {
        cost = cost
            .checked_add(to_score_or_err(rubric.sfb_diagonal())?)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost SFB diagonal".to_string(),
            })?;
    } else if row_diff >= u32::from(rubric.threshold_sfb_long_row_diff().unsigned_abs()) {
        cost = cost
            .checked_add(to_score_or_err(rubric.sfb_long())?)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost SFB long".to_string(),
            })?;
    } else {
        cost = cost
            .checked_add(to_score_or_err(rubric.sfb_base())?)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost SFB base".to_string(),
            })?;
    }
    Ok(cost)
}

fn calculate_non_sfb_penalties(
    rubric: &Rubric,
    k1: &keyforge_model::KeyNode,
    k2: &keyforge_model::KeyNode,
    mut cost: i64,
) -> Result<i64, PhysicsError> {
    let finger_diff = k1.finger.distance(k2.finger);
    let row_diff = (i32::from(k1.row.raw()) - i32::from(k2.row.raw())).unsigned_abs();

    if finger_diff == 1 && k1.finger != FingerIndex::THUMB && k2.finger != FingerIndex::THUMB {
        if row_diff >= u32::from(rubric.threshold_scissor_row_diff().unsigned_abs()) {
            cost = cost
                .checked_add(to_score_or_err(rubric.penalty_scissor())?)
                .ok_or_else(|| PhysicsError::ScoreOverflow {
                    context: "Pair cost scissor".to_string(),
                })?;
        } else if row_diff == 0 {
            let col_diff = (i32::from(k1.col.raw()) - i32::from(k2.col.raw())).unsigned_abs();
            if col_diff > 1 {
                cost = cost
                    .checked_add(to_score_or_err(rubric.sfb_lateral())?)
                    .ok_or_else(|| PhysicsError::ScoreOverflow {
                        context: "Pair cost lateral SFB adjacent".to_string(),
                    })?;
            }
        }
    }
    Ok(cost)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};
    use keyforge_model::KeyNode;
    use std::sync::Arc;

    fn setup_kb_pair() -> Keyboard {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::new(0),
                finger: FingerIndex::new(1),
                row: RowIndex::new(0),
                col: ColIndex::new(0),
                x: 0.0,
                y: 0.0,
                is_home: true,
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::new(0),
                finger: FingerIndex::new(1),
                row: RowIndex::new(1),
                col: ColIndex::new(0),
                x: 0.0,
                y: 1.0,
                is_home: false,
                ..Default::default()
            },
            KeyNode {
                index: 2,
                hand: HandIndex::new(0),
                finger: FingerIndex::new(2),
                row: RowIndex::new(1),
                col: ColIndex::new(1),
                x: 1.0,
                y: 1.0,
                is_home: false,
                ..Default::default()
            },
        ];
        Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap()
    }

    #[test]
    fn test_calculate_pair_cost_sfb() {
        let kb = setup_kb_pair();
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1)).unwrap();
        // Use a direct calculation for the assertion instead of reaching into verify.rs
        let expected_min = (rubric.sfb_base() * 1_000_000.0) as i64;
        assert!(
            cost >= expected_min,
            "SFB should be penalized"
        );
    }

    #[test]
    fn test_calculate_pair_cost_different_hands() {
        let keys = vec![
            KeyNode {
                index: 0,
                hand: HandIndex::new(0),
                finger: FingerIndex::new(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                hand: HandIndex::new(1),
                finger: FingerIndex::new(1),
                ..Default::default()
            },
        ];
        let kb = Arc::new(
            Keyboard::new(keys, keyforge_model::types::RowIndex::new(0), "test".into()).unwrap(),
        );
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1)).unwrap();
        assert_eq!(cost, 0, "Different hands should have 0 cost");
    }

    #[test]
    fn test_calculate_pair_cost_invalid_math() {
        let kb = setup_kb_pair();

        // Force NaN via infinity * 0 or similar if possible, or just inject Infinity
        let rubric = Rubric::builder().travel_lat(f32::INFINITY).build();
        let res = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1));
        assert!(matches!(res, Err(PhysicsError::InvalidInput { .. })));

        let rubric = Rubric::builder().travel_lat(f32::NAN).build();
        let res = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1));
        assert!(matches!(res, Err(PhysicsError::InvalidInput { .. })));
    }

    #[test]
    fn test_calculate_pair_cost_overflows() {
        let kb = setup_kb_pair();

        // 1. checked_add overflow (SFB base)
        let rubric = Rubric::builder().sfb_base(1e20).build();
        let res = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1));
        assert!(matches!(res, Err(PhysicsError::InvalidInput { .. }))); // Score::from_f32 fails first
    }
}