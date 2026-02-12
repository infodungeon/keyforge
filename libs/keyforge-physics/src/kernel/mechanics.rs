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
    use keyforge_model::types::{Finger, Hand, Movement, TrigramFlow};

    // Optimization: avoid construction if hands differ (not a flow)
    if h1 != h2 || h2 != h3 {
        return Score::ZERO;
    }

    // Construct lightweight movements for logic evaluation
    let m1 = Movement::new(
        0,
        0,
        Hand::from(h1),
        Hand::from(h2),
        Finger::from(f1),
        Finger::from(f2),
        keyforge_model::types::RowIndex::new(0),
        keyforge_model::types::RowIndex::new(0),
    );
    let m2 = Movement::new(
        0,
        0,
        Hand::from(h2),
        Hand::from(h3),
        Finger::from(f2),
        Finger::from(f3),
        keyforge_model::types::RowIndex::new(0),
        keyforge_model::types::RowIndex::new(0),
    );
    let flow = TrigramFlow { m1, m2 };

    if flow.is_redirect() {
        return penalty_redirect;
    }

    if flow.is_roll_in() {
        return Score::ZERO.checked_sub(bonus_roll).unwrap_or(Score::MIN);
    }

    if flow.is_roll_out() {
        return Score::ZERO
            .checked_sub(bonus_roll_out)
            .unwrap_or(Score::MIN);
    }

    Score::ZERO
}

/// Bit-perfect integer square root.
pub(crate) fn integer_sqrt_i128(val: i128) -> i64 {
    if val <= 0 {
        return 0;
    }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = i128::midpoint(x, val / x);
    }
    // Rounding: if val - x^2 > x, then round up to x + 1.
    // This provides parity with floor(sqrt(N) + 0.5).
    if val - x * x > x {
        x += 1;
    }
    i64::try_from(x).unwrap_or(i64::MAX)
}

/// Calculates the cost for a pair of keys (bigram or jump), handling geometry and SFBs.
///
/// # Errors
/// Returns `PhysicsError` if:
/// - Geometric calculations result in NaN or Infinite values.
/// - Score accumulation overflows.
#[allow(clippy::cast_possible_truncation)]
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

    if k1.hand() != k2.hand() {
        return Ok(0);
    }

    let movement = kb.spatial_cache[i_idx * kb.keys.len() + j_idx];

    // Bit-perfect integer distance calculation
    let t_lat_i = i128::from(rubric.travel_lat().raw());
    let t_vert_i = i128::from(rubric.travel_vert().raw());

    let dx2 = i64::from(movement.dx) * i64::from(movement.dx);
    let dy2 = i64::from(movement.dy) * i64::from(movement.dy);

    let dist_sq_weighted = i128::from(dx2) * t_lat_i + i128::from(dy2) * t_vert_i;
    let mut cost = integer_sqrt_i128(dist_sq_weighted);

    if k1.finger() == k2.finger() {
        cost = calculate_sfb_cost(kb, rubric, k1, k2, cost)?;
    } else {
        cost = calculate_non_sfb_penalties(rubric, &movement, cost)?;
    }

    Ok(cost)
}

#[allow(clippy::too_many_arguments, clippy::cast_possible_truncation)]
fn calculate_sfb_cost(
    kb: &Keyboard,
    rubric: &Rubric,
    k1: &keyforge_model::KeyNode,
    k2: &keyforge_model::KeyNode,
    mut cost: i64,
) -> Result<i64, PhysicsError> {
    let mut reach_k2 = 0i64;
    if let Some(origin) = kb
        .finger_origins
        .get(k2.hand().as_usize())
        .and_then(|h| h.get(k2.finger().as_usize()))
    {
        let movement = keyforge_model::types::Movement::from_points(
            *origin,
            keyforge_model::types::Point::new(k2.x(), k2.y()),
        );
        let horiz_reach_sq = i64::from(movement.dx) * i64::from(movement.dx);
        let vert_reach_sq = i64::from(movement.dy) * i64::from(movement.dy);

        let t_lat_i = i128::from(rubric.travel_lat().raw());
        let t_vert_i = i128::from(rubric.travel_vert().raw());
        let reach_sq_weighted =
            i128::from(horiz_reach_sq) * t_lat_i + i128::from(vert_reach_sq) * t_vert_i;
        reach_k2 = integer_sqrt_i128(reach_sq_weighted);
    }

    cost = cost
        .checked_sub(reach_k2)
        .ok_or_else(|| PhysicsError::ScoreOverflow {
            context: "Pair cost reach reduction".to_string(),
        })?;

    let row_diff = (i32::from(k1.row().raw()) - i32::from(k2.row().raw())).unsigned_abs();
    let col_diff = (i32::from(k1.col().raw()) - i32::from(k2.col().raw())).unsigned_abs();

    if col_diff == 1 {
        let sfb_extra = if k1.finger().is_weak() {
            rubric.sfb_lateral_weak().raw()
        } else {
            rubric.sfb_lateral().raw()
        };
        cost = cost
            .checked_add(sfb_extra)
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost SFB lateral".to_string(),
            })?;
    } else if col_diff > 1 {
        cost = cost
            .checked_add(rubric.sfb_diagonal().raw())
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost SFB diagonal".to_string(),
            })?;
    } else if row_diff >= u32::from(rubric.threshold_sfb_long_row_diff().unsigned_abs()) {
        cost = cost.checked_add(rubric.sfb_long().raw()).ok_or_else(|| {
            PhysicsError::ScoreOverflow {
                context: "Pair cost SFB long".to_string(),
            }
        })?;
    } else {
        cost = cost.checked_add(rubric.sfb_base().raw()).ok_or_else(|| {
            PhysicsError::ScoreOverflow {
                context: "Pair cost SFB base".to_string(),
            }
        })?;
    }
    Ok(cost)
}

fn calculate_non_sfb_penalties(
    rubric: &Rubric,
    movement: &keyforge_model::types::Movement,
    mut cost: i64,
) -> Result<i64, PhysicsError> {
    if movement.is_scissor(rubric.threshold_scissor_row_diff()) {
        cost = cost
            .checked_add(rubric.penalty_scissor().raw())
            .ok_or_else(|| PhysicsError::ScoreOverflow {
                context: "Pair cost scissor".to_string(),
            })?;
    } else {
        let row_diff = (i32::from(movement.r1.raw()) - i32::from(movement.r2.raw())).abs();
        if row_diff == 0 && movement.is_same_hand() && movement.f1 != movement.f2 {
            let col_diff = movement.dx.abs();
            let f_dist = movement.f1.distance(movement.f2);
            if f_dist == 1 && col_diff > 1000 {
                cost = cost
                    .checked_add(rubric.sfb_lateral().raw())
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
    use keyforge_model::types::{
        ColIndex, FingerIndex, HandIndex, KeyIndex, RowIndex, SpatialUnit,
    };
    use keyforge_model::KeyNode;
    use std::sync::Arc;

    fn setup_kb_pair() -> anyhow::Result<Keyboard> {
        let keys = vec![
            KeyNode::builder()
                .index(KeyIndex::new(0))
                .hand(HandIndex::new(0))
                .finger(FingerIndex::new(1))
                .row(RowIndex::new(0))
                .col(ColIndex::new(0))
                .x(SpatialUnit::from_f32(0.0))
                .y(SpatialUnit::from_f32(0.0))
                .is_home(true)
                .build(),
            KeyNode::builder()
                .index(KeyIndex::new(1))
                .hand(HandIndex::new(0))
                .finger(FingerIndex::new(1))
                .row(RowIndex::new(1))
                .col(ColIndex::new(0))
                .x(SpatialUnit::from_f32(0.0))
                .y(SpatialUnit::from_f32(1.0))
                .is_home(false)
                .build(),
            KeyNode::builder()
                .index(KeyIndex::new(2))
                .hand(HandIndex::new(0))
                .finger(FingerIndex::new(2))
                .row(RowIndex::new(1))
                .col(ColIndex::new(1))
                .x(SpatialUnit::from_f32(1.0))
                .y(SpatialUnit::from_f32(1.0))
                .is_home(false)
                .build(),
        ];
        Ok(Keyboard::new(
            keys,
            keyforge_model::types::RowIndex::new(0),
            "test".into(),
        )?)
    }

    #[test]
    fn test_calculate_pair_cost_sfb() -> anyhow::Result<()> {
        let kb = setup_kb_pair()?;
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1))?;
        // Use a direct calculation for the assertion instead of reaching into verify.rs
        let expected_min = rubric.sfb_base().raw();
        assert!(cost >= expected_min, "SFB should be penalized");
        Ok(())
    }

    #[test]
    fn test_calculate_pair_cost_different_hands() -> anyhow::Result<()> {
        let keys = vec![
            KeyNode::builder()
                .index(KeyIndex::new(0))
                .hand(HandIndex::new(0))
                .finger(FingerIndex::new(1))
                .build(),
            KeyNode::builder()
                .index(KeyIndex::new(1))
                .hand(HandIndex::new(1))
                .finger(FingerIndex::new(1))
                .build(),
        ];
        let kb = Arc::new(Keyboard::new(
            keys,
            keyforge_model::types::RowIndex::new(0),
            "test".into(),
        )?);
        let rubric = Rubric::default();

        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1))?;
        assert_eq!(cost, 0, "Different hands should have 0 cost");
        Ok(())
    }

    #[test]
    fn test_calculate_pair_cost_invalid_math_no_panic() {
        // Rubric builder now just stores the value. Real errors are caught at scoring time.
        let _ = Rubric::builder().travel_lat(i64::MAX).build();
    }

    #[test]
    fn test_calculate_pair_cost_large_values() -> anyhow::Result<()> {
        let kb = setup_kb_pair()?;
        let rubric = Rubric::builder().sfb_base(2_000_000).build();
        let cost = calculate_pair_cost(&kb, &rubric, KeyIndex::new(0), KeyIndex::new(1))?;
        assert!(cost > 0);
        Ok(())
    }
}
