use fastrand::Rng;
use std::collections::HashMap;

// CHANGED: u8 -> u16
pub fn crossover_uniform(
    p1: &[u16],
    p2: &[u16],
    pinned: &[Option<u16>],
    rng: &mut Rng,
) -> Vec<u16> {
    let len = p1.len();
    let mut child = vec![0u16; len];
    let mut filled = vec![false; len];

    // Using HashMap for counts now as 65536 array stack allocation inside this function
    // might be too heavy if called recursively or frequently in threads.
    // However, fast UOX usually needs speed.
    // Let's use a HashMap because layout size is small (30-60 keys).
    // The previous array[256] was fine, array[65536] is 512KB (if usize).
    // 512KB on stack is risky.
    let mut available_counts = HashMap::with_capacity(len);
    for &b in p1 {
        *available_counts.entry(b).or_insert(0) += 1;
    }

    // Enforce Pins
    for (i, &pin) in pinned.iter().enumerate() {
        if i < len {
            if let Some(val) = pin {
                child[i] = val;
                filled[i] = true;
                if let Some(count) = available_counts.get_mut(&val) {
                    if *count > 0 {
                        *count -= 1;
                    }
                }
            }
        }
    }

    // Inherit P1
    for i in 0..len {
        if !filled[i] && rng.bool() {
            let gene = p1[i];
            if let Some(count) = available_counts.get_mut(&gene) {
                if *count > 0 {
                    child[i] = gene;
                    filled[i] = true;
                    *count -= 1;
                }
            }
        }
    }

    // Fill P2
    let mut p2_idx = 0;
    for i in 0..len {
        if !filled[i] {
            while p2_idx < len {
                let gene = p2[p2_idx];
                p2_idx += 1;
                if let Some(count) = available_counts.get_mut(&gene) {
                    if *count > 0 {
                        child[i] = gene;
                        *count -= 1;
                        break;
                    }
                }
            }
        }
    }

    child
}
