use super::types::{DistanceSquared, FingerIndex, HandIndex, KeyIndex};
use keyforge_model::{Keyboard, Rubric};
use std::convert::TryFrom;

/// Calculates the static cost of a transition between two keys.
/// Uses squared distance to penalize long jumps more heavily and avoid sqrt cost.
pub fn calculate_pair_cost(kb: &Keyboard, rubric: &Rubric, i: KeyIndex, j: KeyIndex) -> f32 {
    let i = i.as_usize();
    let j = j.as_usize();

    if i == j {
        return 0.0;
    }

    let k1 = &kb.keys[i];
    let k2 = &kb.keys[j];

    // Lift primitives into Semantic Types
    // In a stricter system, this conversion would happen at the Keyboard constructor boundary.
    // Here, we sanitize on the fly to ensure kernel safety.
    let h1 = HandIndex::try_from(k1.hand).unwrap_or(HandIndex::try_from(0).unwrap());
    let h2 = HandIndex::try_from(k2.hand).unwrap_or(HandIndex::try_from(0).unwrap());

    let f1 = FingerIndex::try_from(k1.finger).unwrap_or(FingerIndex::try_from(1).unwrap());
    let f2 = FingerIndex::try_from(k2.finger).unwrap_or(FingerIndex::try_from(1).unwrap());

    // 1. Squared Distance (Quadratic Penalty)
    // We calculate (Distance^2 * Weight) so that negative weights act as bonuses.
    let dx = (k1.x - k2.x).abs();
    let dy = (k1.y - k2.y).abs();
    let dist_raw = (dx * dx * rubric.travel_lat) + (dy * dy * rubric.travel_vert);
    
    // INVARIANT: kani::assume(dist_raw.is_finite());
    // Guardrail: Enforce non-negative distance cost
    let dist_sq = DistanceSquared::new(dist_raw);
    let mut cost = dist_sq.as_f32();

    // If different hands, we only charge distance (no biomechanical penalty)
    if h1 != h2 {
        return cost;
    }

    // 2. Same Finger Bigram (SFB)
    if f1 == f2 {
        let col_diff = (k1.col - k2.col).abs();
        if col_diff == 1 {
            cost += rubric.sfb_lateral;
        } else {
            cost += rubric.sfb_base;
        }
        return cost;
    }

    // 3. Scissors
    let f1_val = f1.as_u8() as i8;
    let f2_val = f2.as_u8() as i8;
    
    let finger_diff = (f1_val - f2_val).abs();
    let row_diff = (k1.row - k2.row).abs();

    if finger_diff == 1 && row_diff >= 2 {
        cost += rubric.finger_effort[f1.as_usize()];
    }

    // 4. Lateral Stretch
    if row_diff == 0 && finger_diff == 1 {
        let col_dist = (k1.col - k2.col).abs();
        if col_dist > 1 {
            cost += rubric.sfb_lateral;
        }
    }

    cost
}
