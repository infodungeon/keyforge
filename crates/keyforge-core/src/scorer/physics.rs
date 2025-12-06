use keyforge_protocol::config::ScoringWeights;
use keyforge_protocol::geometry::{KeyNode, KeyboardGeometry};
use std::cmp::Ordering;

// --- API RE-EXPORTS ---
pub use super::metrics::reach_cost as get_reach_cost;
pub use super::metrics::weighted_geo_dist as get_geo_dist;

#[derive(Debug, Default, PartialEq)]
pub struct KeyInteraction {
    pub is_same_hand: bool,
    pub finger: usize,
    pub is_strong_finger: bool,

    pub is_repeat: bool,
    pub is_sfb: bool,
    pub is_scissor: bool,
    pub is_lateral_stretch: bool,

    pub is_roll_in: bool,
    pub is_roll_out: bool,

    pub row_diff: i8,
    pub col_diff: i8,
    pub is_home_row: bool,

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

    res.is_outward = k2.row < k1.row;

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

        if k1.row == k2.row && (k1.col - k2.col).abs() == 1 && (k1.is_stretch || k2.is_stretch) {
            res.is_lateral_stretch = true;
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock Helper
    fn make_key(id: usize, hand: u8, finger: u8, row: i8, col: i8) -> KeyNode {
        KeyNode {
            id: id.to_string(),
            hand,
            finger,
            row,
            col,
            x: col as f32,
            y: row as f32,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        }
    }

    #[test]
    fn test_sfb_detection() {
        // Same finger (1), different rows (0 vs 2) -> SFB
        let k1 = make_key(0, 0, 1, 0, 0);
        let k2 = make_key(1, 0, 1, 2, 0);
        let geom = KeyboardGeometry {
            keys: vec![k1, k2],
            ..Default::default()
        };
        let w = ScoringWeights::default();

        let res = analyze_interaction(&geom, 0, 1, &w);
        assert!(res.is_sfb);
        assert_eq!(res.row_diff, 2);
    }

    #[test]
    fn test_scissor_detection() {
        // Adjacent fingers (2 vs 3), steep row diff (0 vs 2) -> Scissor
        let k1 = make_key(0, 0, 2, 0, 2); // Middle Top
        let k2 = make_key(1, 0, 3, 2, 3); // Ring Bottom
        let geom = KeyboardGeometry {
            keys: vec![k1, k2],
            ..Default::default()
        };
        let w = ScoringWeights::default();

        let res = analyze_interaction(&geom, 0, 1, &w);
        assert!(!res.is_sfb);
        assert!(res.is_scissor);
    }

    #[test]
    fn test_roll_logic() {
        // Index (1) -> Ring (3). 1 < 3. Should be Roll Out.
        let k1 = make_key(0, 0, 1, 1, 1);
        let k2 = make_key(1, 0, 3, 1, 3);
        let geom = KeyboardGeometry {
            keys: vec![k1, k2],
            ..Default::default()
        };
        let w = ScoringWeights::default();

        let res = analyze_interaction(&geom, 0, 1, &w);
        assert!(res.is_roll_out);
        assert!(!res.is_roll_in);
    }

    #[test]
    fn test_different_hands_ignored() {
        let k1 = make_key(0, 0, 1, 1, 1); // Left
        let k2 = make_key(1, 1, 1, 1, 8); // Right
        let geom = KeyboardGeometry {
            keys: vec![k1, k2],
            ..Default::default()
        };
        let w = ScoringWeights::default();

        let res = analyze_interaction(&geom, 0, 1, &w);
        assert!(!res.is_same_hand);
        assert!(!res.is_sfb); // Cannot be SFB if diff hands
    }
}
