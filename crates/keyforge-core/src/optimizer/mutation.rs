use crate::config::LayoutDefinitions;
use crate::geometry::KeyboardGeometry;
use fastrand::Rng;

pub fn generate_tiered_layout(
    rng: &mut Rng,
    defs: &LayoutDefinitions,
    geom: &KeyboardGeometry,
    size: usize,
    pinned: &[Option<u8>], // NEW ARGUMENT
) -> Vec<u8> {
    let mut layout = vec![0u8; size];

    // 1. Fill Pinned Keys first and remove them from pools
    let mut pinned_chars = [false; 256];

    for (i, &p) in pinned.iter().enumerate() {
        if i < size {
            if let Some(c) = p {
                layout[i] = c;
                pinned_chars[c as usize] = true;
            }
        }
    }

    // Helper to filter pools
    let filter_pool = |src: &str| -> Vec<u8> {
        src.as_bytes()
            .iter()
            .cloned()
            .filter(|&c| !pinned_chars[c as usize])
            .collect()
    };

    let mut high = filter_pool(&defs.tier_high_chars);
    let mut med = filter_pool(&defs.tier_med_chars);
    let mut low = filter_pool(&defs.tier_low_chars);

    rng.shuffle(&mut high);
    rng.shuffle(&mut med);
    rng.shuffle(&mut low);

    // 2. Fill Prime Slots (skip if pinned)
    for &slot in &geom.prime_slots {
        if slot < size && layout[slot] == 0 {
            if let Some(c) = high.pop() {
                layout[slot] = c;
            } else if let Some(c) = med.pop() {
                layout[slot] = c;
            } else {
                layout[slot] = 0;
            }
        }
    }

    // 3. Fill Medium Slots
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

    // 4. Fill Low Slots
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
