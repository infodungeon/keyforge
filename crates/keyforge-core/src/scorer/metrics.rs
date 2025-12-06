use keyforge_protocol::geometry::KeyboardGeometry;

#[inline(always)]
pub fn euclidean_dist(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    (dx * dx + dy * dy).sqrt()
}

#[inline(always)]
pub fn weighted_geo_dist(
    geom: &KeyboardGeometry,
    i: usize,
    j: usize,
    lat_weight: f32,
    vert_weight: f32,
) -> f32 {
    if i == j {
        return 0.0;
    }
    // Safety check for bounds
    if i >= geom.keys.len() || j >= geom.keys.len() {
        return 0.0;
    }

    let k1 = &geom.keys[i];
    let k2 = &geom.keys[j];

    // Distance is 0 if on different hands (no physical travel)
    if k1.hand != k2.hand {
        return 0.0;
    }

    let dx = (k1.x - k2.x).abs() * lat_weight;
    let dy = (k1.y - k2.y).abs() * vert_weight;

    (dx * dx + dy * dy).sqrt()
}

#[inline(always)]
pub fn reach_cost(geom: &KeyboardGeometry, i: usize, lat_weight: f32, vert_weight: f32) -> f32 {
    if i >= geom.keys.len() {
        return 0.0;
    }
    let k = &geom.keys[i];
    // Safety check for array bounds
    if k.hand as usize >= 2 || k.finger as usize >= 5 {
        return 0.0;
    }

    let (hx, hy) = geom.finger_origins[k.hand as usize][k.finger as usize];

    let dx = (k.x - hx).abs() * lat_weight;
    let dy = (k.y - hy).abs() * vert_weight;

    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_euclidean_basic() {
        // 3-4-5 Triangle
        let d = euclidean_dist(0.0, 0.0, 3.0, 4.0);
        assert!((d - 5.0).abs() < 0.001, "Expected 5.0, got {}", d);
    }

    proptest! {
        #[test]
        fn prop_euclidean_symmetry(x1 in -100.0f32..100.0, y1 in -100.0f32..100.0, x2 in -100.0f32..100.0, y2 in -100.0f32..100.0) {
            let d1 = euclidean_dist(x1, y1, x2, y2);
            let d2 = euclidean_dist(x2, y2, x1, y1);
            prop_assert!((d1 - d2).abs() < 0.0001);
        }

        #[test]
        fn prop_euclidean_non_negative(x1 in -100.0f32..100.0, y1 in -100.0f32..100.0, x2 in -100.0f32..100.0, y2 in -100.0f32..100.0) {
            let d = euclidean_dist(x1, y1, x2, y2);
            prop_assert!(d >= 0.0);
        }
    }
}
