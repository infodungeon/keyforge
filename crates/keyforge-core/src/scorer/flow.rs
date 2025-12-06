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
            res.is_inward_roll = true;
        } else {
            res.is_outward_roll = true;
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(finger: u8, hand: u8) -> KeyNode {
        KeyNode {
            id: "".into(),
            hand,
            finger,
            row: 0,
            col: 0,
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        }
    }

    #[test]
    fn test_redirect_detection() {
        // Index(1) -> Ring(3) -> Middle(2)
        // 1->3 (+) then 3->2 (-) => Redirect
        let k1 = make_key(1, 0);
        let k2 = make_key(3, 0);
        let k3 = make_key(2, 0);

        let res = analyze_flow(&k1, &k2, &k3);
        assert!(res.is_redirect);
        assert!(!res.is_inward_roll);
    }

    #[test]
    fn test_inward_roll() {
        // Ring(3) -> Middle(2) -> Index(1)
        // 3->2 (-) then 2->1 (-) => Inward
        let k1 = make_key(3, 0);
        let k2 = make_key(2, 0);
        let k3 = make_key(1, 0);

        let res = analyze_flow(&k1, &k2, &k3);
        assert!(res.is_inward_roll);
        assert!(!res.is_redirect);
    }

    #[test]
    fn test_skipgram() {
        // Index(1) -> Middle(2) -> Index(1)
        // Same finger (1) with one gap
        let k1 = make_key(1, 0);
        let k2 = make_key(2, 0);
        let k3 = make_key(1, 0);

        let res = analyze_flow(&k1, &k2, &k3);
        assert!(res.is_skip);
        // Note: ABA is also a redirect (Change of direction)
        assert!(res.is_redirect);
    }
}
