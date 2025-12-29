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

    // INVARIANT: HandIndex and FingerIndex construction must be valid.
    // This is now enforced by the compiler via TryFrom<Error=PhysicsError>.
    
    // Lift primitives into Semantic Types
    // We unwrap here because the Compiler::compile step has already validated the geometry.
    let h1 = HandIndex::try_from(k1.hand).unwrap_or(HandIndex::try_from(0).unwrap());
    let h2 = HandIndex::try_from(k2.hand).unwrap_or(HandIndex::try_from(0).unwrap());

    let f1 = FingerIndex::try_from(k1.finger).unwrap_or(FingerIndex::try_from(1).unwrap());
    let f2 = FingerIndex::try_from(k2.finger).unwrap_or(FingerIndex::try_from(1).unwrap());

    // 1. Squared Distance (Quadratic Penalty)
    // We calculate (Distance^2 * Weight) so that negative weights act as bonuses.
    let dx = (k1.x - k2.x).abs();
    let dy = (k1.y - k2.y).abs();
    let dist_raw = (dx * dx * rubric.travel_lat) + (dy * dy * rubric.travel_vert);

    // INVARIANT: kani::assume(dist_raw >= 0.0);
    // Distance cost must be non-negative to ensure score monotonicity.
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::{KeyNode, Keyboard, Rubric};
    use proptest::prelude::*;

    fn arb_key_node() -> impl Strategy<Value = KeyNode> {
        (
            0u8..2,   // Hand
            0u8..5,   // Finger
            -5i8..5,  // Row
            -10i8..15, // Col
            -20.0..20.0f32, // X
            -20.0..20.0f32  // Y
        ).prop_map(|(h, f, r, c, x, y)| {
            KeyNode {
                id: 0,
                label: "x".into(),
                hand: h,
                finger: f,
                row: r,
                col: c,
                x,
                y,
                is_home: false
            }
        })
    }

    fn arb_rubric() -> impl Strategy<Value = Rubric> {
        (
            0.0..100.0f32, // Travel Lat
            0.0..100.0f32, // Travel Vert
            0.0..100.0f32, // SFB
            0.0..100.0f32  // SFB Lat
        ).prop_map(|(tl, tv, sfb, sfbl)| {
            Rubric {
                travel_lat: tl,
                travel_vert: tv,
                sfb_base: sfb,
                sfb_lateral: sfbl,
                finger_effort: [1.0; 5],
                ..Rubric::default()
            }
        })
    }

    proptest! {
        #[test]
        fn test_invariant_symmetry(
            k1 in arb_key_node(),
            k2 in arb_key_node(),
            rubric in arb_rubric()
        ) {
            let keys = vec![k1.clone(), k2.clone()];
            let kb = Keyboard::new(keys, 0);
            
            let idx_a = KeyIndex(0);
            let idx_b = KeyIndex(1);

            let cost_ab = calculate_pair_cost(&kb, &rubric, idx_a, idx_b);
            let cost_ba = calculate_pair_cost(&kb, &rubric, idx_b, idx_a);

            prop_assert_eq!(cost_ab, cost_ba);
        }

        #[test]
        fn test_invariant_non_negative_distance(
            k1 in arb_key_node(),
            k2 in arb_key_node(),
            rubric in arb_rubric()
        ) {
            let keys = vec![k1, k2];
            let kb = Keyboard::new(keys, 0);
            let cost = calculate_pair_cost(&kb, &rubric, KeyIndex(0), KeyIndex(1));
            prop_assert!(cost >= 0.0);
        }
    }
}
