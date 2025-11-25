use crate::geometry::KeyNode;

#[derive(Debug, Default)]
pub struct FlowAnalysis {
    pub is_3_hand_run: bool,
    pub is_skip: bool,      // ABA pattern on same hand (e.g. Index -> Middle -> Index)
    pub is_redirect: bool,  // Direction change (e.g. Pinky -> Index -> Middle)
    pub is_inward_roll: bool, // Pinky -> Ring -> Middle (Decreasing finger index)
    pub is_outward_roll: bool, // Index -> Middle -> Ring (Increasing finger index)
}

pub fn analyze_flow(k1: &KeyNode, k2: &KeyNode, k3: &KeyNode) -> FlowAnalysis {
    let mut res = FlowAnalysis::default();

    // Must be a 3-key run on the same hand
    if k1.hand != k2.hand || k2.hand != k3.hand {
        return res;
    }
    res.is_3_hand_run = true;

    let f1 = k1.finger as i8;
    let f2 = k2.finger as i8;
    let f3 = k3.finger as i8;

    // 1. Check Skipgram (same finger, separated by one)
    if f1 == f3 && f1 != f2 {
        res.is_skip = true;
    }

    let dir1 = f2 - f1;
    let dir2 = f3 - f2;

    // 2. Check Directional Flow (requires movement between all keys)
    if dir1 != 0 && dir2 != 0 {
        if dir1.signum() != dir2.signum() {
            res.is_redirect = true;
        } else {
            // Monotonic direction (Rolls)
            // Standard numbering: 0=Thumb, 1=Index ... 4=Pinky
            // Pinky(4) -> Index(1) is decreasing (< 0). This is INWARD.
            if dir1 < 0 {
                res.is_inward_roll = true;
            } else {
                res.is_outward_roll = true;
            }
        }
    }

    res
}