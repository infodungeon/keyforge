use keyforge_protocol::geometry::KeyNode;

#[derive(Debug, Default)]
pub struct FlowAnalysis {
    pub is_3_hand_run: bool,
    pub is_skip: bool,
    pub is_redirect: bool,
    pub is_inward_roll: bool,
    pub is_outward_roll: bool,
}

pub fn analyze_flow(k1: &KeyNode, k2: &KeyNode, k3: &KeyNode) -> FlowAnalysis {
    let mut res = FlowAnalysis::default();

    if k1.hand != k2.hand || k2.hand != k3.hand {
        return res;
    }
    res.is_3_hand_run = true;

    let f1 = k1.finger as i8;
    let f2 = k2.finger as i8;
    let f3 = k3.finger as i8;

    if f1 == f3 && f1 != f2 {
        res.is_skip = true;
    }

    let dir1 = f2 - f1;
    let dir2 = f3 - f2;

    if dir1 != 0 && dir2 != 0 {
        if dir1.signum() != dir2.signum() {
            res.is_redirect = true;
        } else if dir1 < 0 {
            // Collapsed else if
            res.is_inward_roll = true;
        } else {
            res.is_outward_roll = true;
        }
    }

    res
}
