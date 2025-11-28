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
    pub is_outward: bool, // Geometric extension (for SFBs)
}

/// Calculates pure Euclidean distance between two keys using physical coordinates.
pub fn get_geo_dist(geom: &KeyboardGeometry, i: usize, j: usize, scale: f32) -> f32 {
    if i == j {
        return 0.0;
    }
    let k1 = &geom.keys[i];
    let k2 = &geom.keys[j];
    if k1.hand != k2.hand {
        return 0.0;
    }
    let dx = k1.x - k2.x;
    let dy = k1.y - k2.y;
    (dx * dx + dy * dy).sqrt() * scale
}

/// Calculates distance from Home Position (Reach).
pub fn get_reach_cost(geom: &KeyboardGeometry, i: usize, scale: f32) -> f32 {
    let ki = &geom.keys[i];
    // Home Row is defined as 1 in standard/ortho layouts
    let dy = (ki.row - 1).abs() as f32;
    let mut dx = 0.0;

    if ki.finger == 1 && ki.is_stretch {
        dx = 1.0;
    }

    let dist = (dx * dx + dy * dy).sqrt();
    dist * scale
}

pub fn analyze_interaction(geom: &KeyboardGeometry, i: usize, j: usize) -> KeyInteraction {
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
        res.is_home_row = k1.row == 1;
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
        // Bottom Lateral Sequence (Rank 9 DV)
        if k1.row == 2 && k2.row == 2 && res.col_diff > 0 {
            res.is_bot_lat_seq = true;
        }
    } else {
        // Different Fingers = Potential Roll or Scissor or Lat Stretch

        // 1. Bigram Roll Detection
        match k1.finger.cmp(&k2.finger) {
            Ordering::Greater => res.is_roll_in = true,
            Ordering::Less => res.is_roll_out = true,
            Ordering::Equal => {}
        }

        // 2. Scissor
        if (k1.finger as i8 - k2.finger as i8).abs() == 1 && (k1.row - k2.row).abs() >= 2 {
            res.is_scissor = true;
        }

        // 3. Lateral Stretch (Non-SFB)
        if k1.row == k2.row && (k1.col - k2.col).abs() == 1 && (k1.is_stretch || k2.is_stretch) {
            res.is_lateral_stretch = true;
        }
    }

    // Geometric Direction (for SFB nuances)
    if k2.row < k1.row {
        res.is_outward = true;
    }

    if k1.is_stretch && !k2.is_stretch {
        res.is_outward = false;
    }
    if !k1.is_stretch && k2.is_stretch {
        res.is_outward = true;
    }

    res
}
