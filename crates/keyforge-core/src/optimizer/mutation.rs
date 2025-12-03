use crate::config::LayoutDefinitions;
use crate::geometry::KeyboardGeometry;
use fastrand::Rng;

pub fn generate_tiered_layout(
    rng: &mut Rng,
    defs: &LayoutDefinitions,
    geom: &KeyboardGeometry,
    size: usize,
    pinned: &[Option<u16>], // CHANGED: u8 -> u16
) -> Vec<u16> {
    // CHANGED
    let mut layout = vec![0u16; size];

    // 1. Fill Pinned Keys
    // We use a simple boolean array for standard ASCII tracking (0-255)
    // For anything higher, we assume it's not part of the "scrambling pool"
    let mut pinned_chars = [false; 256];

    for (i, &p) in pinned.iter().enumerate() {
        if i < size {
            if let Some(c) = p {
                layout[i] = c;
                if c < 256 {
                    pinned_chars[c as usize] = true;
                }
            }
        }
    }

    // Helper to filter pools (only applies to standard ASCII chars)
    let filter_pool = |src: &str| -> Vec<u16> {
        src.as_bytes()
            .iter()
            .map(|&b| b as u16)
            .filter(|&c| c >= 256 || !pinned_chars[c as usize])
            .collect()
    };

    let mut high = filter_pool(&defs.tier_high_chars);
    let mut med = filter_pool(&defs.tier_med_chars);
    let mut low = filter_pool(&defs.tier_low_chars);

    rng.shuffle(&mut high);
    rng.shuffle(&mut med);
    rng.shuffle(&mut low);

    let mut fill_slot = |slot: usize, pools: &mut [&mut Vec<u16>]| {
        if slot < size && layout[slot] == 0 {
            for pool in pools {
                if let Some(c) = pool.pop() {
                    layout[slot] = c;
                    return;
                }
            }
            layout[slot] = 0;
        }
    };

    for &slot in &geom.prime_slots {
        fill_slot(slot, &mut [&mut high, &mut med]);
    }
    for &slot in &geom.med_slots {
        fill_slot(slot, &mut [&mut med, &mut low]);
    }
    for &slot in &geom.low_slots {
        fill_slot(slot, &mut [&mut low, &mut med, &mut high]);
    }

    layout
}

// CHANGED: Returns Box<[u8; 65536]>
pub fn build_pos_map(layout: &[u16]) -> Box<[u8; 65536]> {
    let mut map = Box::new([255u8; 65536]);
    for (i, &code) in layout.iter().enumerate() {
        if code != 0 {
            // 0 is KC_NO
            map[code as usize] = i as u8;

            // Handle Case Insensitivity for standard ASCII
            if code < 128 {
                let b = code as u8;
                if b.is_ascii_uppercase() {
                    map[b.to_ascii_lowercase() as usize] = i as u8;
                } else if b.is_ascii_lowercase() {
                    map[b.to_ascii_uppercase() as usize] = i as u8;
                }
            }
        }
    }
    map
}

pub fn fails_sanity(
    pos_map: &[u8; 65536], // CHANGED signature
    critical_bigrams: &[[u8; 2]],
    geom: &KeyboardGeometry,
) -> bool {
    for pair in critical_bigrams {
        let p1 = pos_map[pair[0] as usize];
        let p2 = pos_map[pair[1] as usize];

        if p1 == 255 || p2 == 255 {
            continue;
        }

        let info1 = &geom.keys[p1 as usize];
        let info2 = &geom.keys[p2 as usize];

        if info1.hand == info2.hand && info1.finger == info2.finger {
            return true;
        }
    }
    false
}
