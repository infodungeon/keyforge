use crate::config::ScoringWeights;
use crate::geometry::KeyboardGeometry;
use std::cmp::Ordering;

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

/// Calculates pure Euclidean distance between two keys using physical coordinates.
pub fn get_geo_dist(
    geom: &KeyboardGeometry,
    i: usize,
    j: usize,
    lat_weight: f32,
    vert_weight: f32,
) -> f32 {
    if i == j {
        return 0.0;
    }
    let k1 = &geom.keys[i];
    let k2 = &geom.keys[j];
    if k1.hand != k2.hand {
        return 0.0;
    }

    let dx = (k1.x - k2.x).abs();
    let dy = (k1.y - k2.y).abs();

    let weighted_x = dx * lat_weight;
    let weighted_y = dy * vert_weight;

    (weighted_x * weighted_x + weighted_y * weighted_y).sqrt()
}

/// Calculates distance from Home Position (Reach).
pub fn get_reach_cost(geom: &KeyboardGeometry, i: usize, lat_weight: f32, vert_weight: f32) -> f32 {
    let ki = &geom.keys[i];

    // Get the home position for this specific hand and finger
    let (home_x, home_y) = geom.finger_origins[ki.hand as usize][ki.finger as usize];

    let dx = (ki.x - home_x).abs();
    let dy = (ki.y - home_y).abs();

    let weighted_x = dx * lat_weight;
    let weighted_y = dy * vert_weight;

    (weighted_x * weighted_x + weighted_y * weighted_y).sqrt()
}

pub fn analyze_interaction(
    geom: &KeyboardGeometry,
    i: usize,
    j: usize,
    weights: &ScoringWeights,
) -> KeyInteraction {
    let k1 = &geom.keys[i];
    let k2 = &geom.keys[j];
    let mut res = KeyInteraction::default();

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

    if k1.finger == k2.finger {
        res.is_sfb = true;
        res.row_diff = (k1.row - k2.row).abs();
        res.col_diff = (k1.col - k2.col).abs();

        if res.row_diff == 0 && res.col_diff == 1 {
            res.is_lat_step = true;
        }

        // Bottom Lateral Sequence (Usually Row 2 <-> Row 2/3 Lateral)
        if k1.row > geom.home_row && k2.row > geom.home_row && res.col_diff > 0 {
            res.is_bot_lat_seq = true;
        }
    } else {
        // Different Fingers
        match k1.finger.cmp(&k2.finger) {
            Ordering::Greater => res.is_roll_in = true,
            Ordering::Less => res.is_roll_out = true,
            Ordering::Equal => {}
        }

        // Scissor Detection
        // Use configurable threshold
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

        // Lateral Stretch (Non-SFB)
        if k1.row == k2.row && (k1.col - k2.col).abs() == 1 && (k1.is_stretch || k2.is_stretch) {
            res.is_lateral_stretch = true;
        }
    }

    // Geometric Direction
    if k2.row < k1.row {
        res.is_outward = true;
    }

    // Lateral Index Logic
    if k1.is_stretch && !k2.is_stretch {
        res.is_outward = false;
    }
    if !k1.is_stretch && k2.is_stretch {
        res.is_outward = true;
    }

    res
}
