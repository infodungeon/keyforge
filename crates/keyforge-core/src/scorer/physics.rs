use crate::config::ScoringWeights;
use crate::geometry::{KeyNode, KeyboardGeometry};
use std::cmp::Ordering;

// --- API RE-EXPORTS ---
pub use super::metrics::reach_cost as get_reach_cost;
pub use super::metrics::weighted_geo_dist as get_geo_dist;

#[derive(Debug, Default, PartialEq)]
pub struct KeyInteraction {
    pub is_same_hand: bool,
    pub finger: usize,
    pub is_strong_finger: bool,

    // Interaction Types
    pub is_repeat: bool,
    pub is_sfb: bool,
    pub is_scissor: bool,
    pub is_lateral_stretch: bool,

    // Roll Analysis (Bigram)
    pub is_roll_in: bool,
    pub is_roll_out: bool,

    // Geometric Details
    pub row_diff: i8,
    pub col_diff: i8,
    pub is_home_row: bool,

    // Nuances
    pub is_lat_step: bool,
    pub is_stretch_col: bool,
    pub is_bot_lat_seq: bool,
    pub is_outward: bool,
}

#[inline(always)]
fn check_sfb(res: &mut KeyInteraction, k1: &KeyNode, k2: &KeyNode) {
    res.is_sfb = true;
    res.row_diff = (k1.row - k2.row).abs();
    res.col_diff = (k1.col - k2.col).abs();

    if res.row_diff == 0 && res.col_diff == 1 {
        res.is_lat_step = true;
    }
    // Bottom Lateral Sequence (Usually Row 2 <-> Row 2/3 Lateral)
    if k1.row > 1 && k2.row > 1 && res.col_diff > 0 {
        res.is_bot_lat_seq = true;
    }
}

#[inline(always)]
fn check_scissors(res: &mut KeyInteraction, k1: &KeyNode, k2: &KeyNode, weights: &ScoringWeights) {
    if (k1.finger as i8 - k2.finger as i8).abs() == 1
        && (k1.row - k2.row).abs() >= weights.threshold_scissor_row_diff
    {
        res.is_scissor = true;

        // --- DYNAMIC SCISSOR EXCEPTION ---
        let (top_finger, bot_finger) = if k1.row < k2.row {
            (k1.finger, k2.finger)
        } else {
            (k2.finger, k1.finger)
        };

        let comfy_list = weights.get_comfortable_scissors();
        for (c_top, c_bot) in comfy_list {
            if top_finger == c_top && bot_finger == c_bot {
                res.is_scissor = false;
                break;
            }
        }
    }
}

#[inline(always)]
fn check_rolls(res: &mut KeyInteraction, k1: &KeyNode, k2: &KeyNode) {
    match k1.finger.cmp(&k2.finger) {
        Ordering::Greater => res.is_roll_in = true,
        Ordering::Less => res.is_roll_out = true,
        Ordering::Equal => {}
    }
}

pub fn analyze_interaction(
    geom: &KeyboardGeometry,
    i: usize,
    j: usize,
    weights: &ScoringWeights,
) -> KeyInteraction {
    let mut res = KeyInteraction::default();

    // Safety check for bounds
    if i >= geom.keys.len() || j >= geom.keys.len() {
        return res;
    }

    let k1 = &geom.keys[i];
    let k2 = &geom.keys[j];

    if k1.hand != k2.hand {
        return res;
    }
    res.is_same_hand = true;
    res.finger = k1.finger as usize;
    res.is_strong_finger = res.finger == 1 || res.finger == 2;

    if i == j {
        res.is_repeat = true;
        res.is_home_row = k1.row == geom.home_row;
        res.is_stretch_col = k1.is_stretch;
        return res;
    }

    // Geometric Direction
    res.is_outward = k2.row < k1.row;

    // Lateral Index Logic
    if k1.is_stretch && !k2.is_stretch {
        res.is_outward = false;
    }
    if !k1.is_stretch && k2.is_stretch {
        res.is_outward = true;
    }

    if k1.finger == k2.finger {
        check_sfb(&mut res, k1, k2);
    } else {
        check_rolls(&mut res, k1, k2);
        check_scissors(&mut res, k1, k2, weights);

        // Lateral Stretch (Non-SFB)
        // REVERTED: Ensure keys are adjacent (col_diff == 1).
        // This prevents Pinky -> Center Key jumps (like N->G in Graphite) from
        // being penalized as "Lateral Stretch", which should only apply to
        // Index -> Center Key or awkward adjacent spreads.
        if k1.row == k2.row && (k1.col - k2.col).abs() == 1 && (k1.is_stretch || k2.is_stretch) {
            res.is_lateral_stretch = true;
        }
    }

    res
}
