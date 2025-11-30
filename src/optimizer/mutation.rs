use crate::config::LayoutDefinitions;
use crate::geometry::KeyboardGeometry;
use fastrand::Rng;

pub fn generate_tiered_layout(
    rng: &mut Rng,
    defs: &LayoutDefinitions,
    geom: &KeyboardGeometry,
    size: usize,
) -> Vec<u8> {
    let mut layout = vec![0u8; size];

    let mut high = defs.tier_high_chars.as_bytes().to_vec();
    let mut med = defs.tier_med_chars.as_bytes().to_vec();
    let mut low = defs.tier_low_chars.as_bytes().to_vec();

    rng.shuffle(&mut high);
    rng.shuffle(&mut med);
    rng.shuffle(&mut low);

    // 1. Fill Prime Slots
    for &slot in &geom.prime_slots {
        if slot < size {
            if let Some(c) = high.pop() {
                layout[slot] = c;
            } else if let Some(c) = med.pop() {
                layout[slot] = c;
            } else {
                layout[slot] = 0;
            }
        }
    }

    // 2. Fill Medium Slots
    for &slot in &geom.med_slots {
        if slot < size && layout[slot] == 0 {
            if let Some(c) = med.pop() {
                layout[slot] = c;
            } else if let Some(c) = low.pop() {
                layout[slot] = c;
            } else {
                layout[slot] = 0;
            }
        }
    }

    // 3. Fill Low Slots
    for &slot in &geom.low_slots {
        if slot < size && layout[slot] == 0 {
            if let Some(c) = low.pop() {
                layout[slot] = c;
            } else if let Some(c) = med.pop() {
                layout[slot] = c;
            } else if let Some(c) = high.pop() {
                layout[slot] = c;
            } else {
                layout[slot] = 0;
            }
        }
    }

    layout
}

pub fn build_pos_map(layout: &[u8]) -> [u8; 256] {
    let mut map = [255u8; 256];
    for (i, &byte) in layout.iter().enumerate() {
        if byte != 0 {
            map[byte as usize] = i as u8;
            if byte.is_ascii_uppercase() {
                map[byte.to_ascii_lowercase() as usize] = i as u8;
            } else if byte.is_ascii_lowercase() {
                map[byte.to_ascii_uppercase() as usize] = i as u8;
            }
        }
    }
    map
}

pub fn fails_sanity(
    pos_map: &[u8; 256],
    critical_bigrams: &[[u8; 2]],
    geom: &KeyboardGeometry,
) -> bool {
    for pair in critical_bigrams {
        let p1 = pos_map[pair[0] as usize];
        let p2 = pos_map[pair[1] as usize];

        if p1 == 255 || p2 == 255 {
            continue;
        }

        let p1_idx = p1 as usize;
        let p2_idx = p2 as usize;

        if p1_idx >= geom.keys.len() || p2_idx >= geom.keys.len() {
            continue;
        }

        let info1 = &geom.keys[p1_idx];
        let info2 = &geom.keys[p2_idx];

        if info1.hand == info2.hand && info1.finger == info2.finger {
            return true;
        }
    }
    false
}
