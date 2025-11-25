use crate::geometry::KeyboardGeometry;

#[derive(Debug, Default, PartialEq)]
pub struct KeyInteraction {
    pub is_same_hand: bool,
    pub finger: usize,          // 0=Thumb, 1=Index, ... 4=Pinky
    pub is_strong_finger: bool, // Index or Middle

    // Interaction Type
    pub is_repeat: bool,          // SFR
    pub is_sfb: bool,             // SFB
    pub is_scissor: bool,         // Adjacent finger row jump
    pub is_lateral_stretch: bool, // Non-SFB lateral

    // Geometric Details
    pub row_diff: i8,      // Abs row difference
    pub col_diff: i8,      // Abs col difference
    pub is_home_row: bool, // Are we on Row 1? (For SFRs)

    // Nuances for SFBs/SFRs
    pub is_lat_step: bool,    // Lateral step (Same Row, Col diff 1)
    pub is_stretch_col: bool, // Are we in the lateral column?
    pub is_bot_lat_seq: bool, // Sequence involves Bottom and Bottom-Lateral
    pub is_outward: bool,     // Extension (Bad) vs Inward/Flexion (Good)
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

pub fn get_reach_cost(geom: &KeyboardGeometry, i: usize, scale: f32) -> f32 {
    let ki = &geom.keys[i];
    // Home Row is defined as 1
    let dy = (ki.row - 1).abs() as f32;

    let mut dx = 0.0;
    if ki.finger == 1 {
        if ki.is_stretch {
            dx = 1.0;
        }
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

    // Check vertical direction (Assuming Row 0=Top, 1=Home, 2=Bot)
    // Row 1->2 (Flexion/Inward), Row 1->0 (Extension/Outward)
    // Note: This is simplified. Moving Bot->Home is Extension.
    if k2.row < k1.row {
        res.is_outward = true;
    } // Moving Up (Ext)
      // We don't explicitly flag inward, default is neutral/inward.

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

        // Lateral Step: Same Row, Adjacent Column
        // Covers Rank 4 (TG)
        if res.row_diff == 0 && res.col_diff == 1 {
            res.is_lat_step = true;
        }

        // Bottom Lateral Sequence: Covers Rank 9 (DV)
        // Both on Bottom Row, Col diff > 0
        if k1.row == 2 && k2.row == 2 && res.col_diff > 0 {
            res.is_bot_lat_seq = true;
        }
    } else {
        // Non-SFB Checks
        if k1.row == k2.row && (k1.col - k2.col).abs() == 1 {
            if k1.is_stretch || k2.is_stretch {
                res.is_lateral_stretch = true;
            }
        }

        if (k1.finger as i8 - k2.finger as i8).abs() == 1 && (k1.row - k2.row).abs() >= 2 {
            res.is_scissor = true;
        }
    }

    res
}
