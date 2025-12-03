use crate::geometry::KeyboardGeometry;

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
    let k1 = &geom.keys[i];
    let k2 = &geom.keys[j];

    // Quick exit for different hands
    if k1.hand != k2.hand {
        return 0.0;
    }

    let dx = (k1.x - k2.x).abs() * lat_weight;
    let dy = (k1.y - k2.y).abs() * vert_weight;

    (dx * dx + dy * dy).sqrt()
}

#[inline(always)]
pub fn reach_cost(geom: &KeyboardGeometry, i: usize, lat_weight: f32, vert_weight: f32) -> f32 {
    let k = &geom.keys[i];
    let (hx, hy) = geom.finger_origins[k.hand as usize][k.finger as usize];

    let dx = (k.x - hx).abs() * lat_weight;
    let dy = (k.y - hy).abs() * vert_weight;

    (dx * dx + dy * dy).sqrt()
}
