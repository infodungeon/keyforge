use keyforge_model::{Keyboard, Rubric};

/// Calculates the static cost of a transition between two keys.
/// Uses squared distance to penalize long jumps more heavily and avoid sqrt cost.
pub fn calculate_pair_cost(kb: &Keyboard, rubric: &Rubric, i: usize, j: usize) -> f32 {
    if i == j {
        return 0.0;
    }

    let k1 = &kb.keys[i];
    let k2 = &kb.keys[j];

    // 1. Squared Distance (Quadratic Penalty)
    let dx = (k1.x - k2.x).abs() * rubric.travel_lat;
    let dy = (k1.y - k2.y).abs() * rubric.travel_vert;
    let dist_sq = dx * dx + dy * dy;

    let mut cost = dist_sq;

    // If different hands, we only charge distance (no biomechanical penalty)
    if k1.hand != k2.hand {
        return cost;
    }

    // 2. Same Finger Bigram (SFB)
    if k1.finger == k2.finger {
        let col_diff = (k1.col - k2.col).abs();
        if col_diff == 1 {
            cost += rubric.sfb_lateral;
        } else {
            cost += rubric.sfb_base;
        }
        return cost;
    }

    // 3. Scissors
    let finger_diff = (k1.finger as i8 - k2.finger as i8).abs();
    let row_diff = (k1.row - k2.row).abs();

    if finger_diff == 1 && row_diff >= 2 {
        // P2 FIX: Removed hardcoded * 10.0 multiplier.
        // Now relies purely on the configured finger effort weight.
        cost += rubric.finger_effort[k1.finger as usize];
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
