use fastrand::Rng;

// Optimized Uniform Crossover (UOX)
// Preserves the exact multiset of characters from parents without using HashMaps.
pub fn crossover_uniform(
    p1: &[u16],
    p2: &[u16],
    pinned: &[Option<u16>],
    rng: &mut Rng,
) -> Vec<u16> {
    let len = p1.len();
    let mut child = vec![0u16; len];
    let mut filled = vec![false; len];

    // 1. Apply Pins (Dominant)
    for (i, &pin) in pinned.iter().enumerate() {
        if i < len {
            if let Some(val) = pin {
                child[i] = val;
                filled[i] = true;
            }
        }
    }

    // 2. Build Frequency Table of what is *needed*
    // We calculate what is in P1. The child must end up with exactly these keys.
    // Since u16 range is large, we can't use a simple array.
    // However, N is small (~40). A simple vector is faster than a HashMap here.
    // format: (code, count)
    let mut needed: Vec<(u16, u8)> = Vec::with_capacity(len);

    for &code in p1 {
        if let Some(entry) = needed.iter_mut().find(|(c, _)| *c == code) {
            entry.1 += 1;
        } else {
            needed.push((code, 1));
        }
    }

    // Subtract pins from needs
    for (i, &is_f) in filled.iter().enumerate() {
        if is_f {
            let code = child[i];
            if let Some(entry) = needed.iter_mut().find(|(c, _)| *c == code) {
                if entry.1 > 0 {
                    entry.1 -= 1;
                }
            }
        }
    }

    // 3. Inherit from P1 (Probabilistic)
    for i in 0..len {
        if !filled[i] && rng.bool() {
            let gene = p1[i];
            if let Some(entry) = needed.iter_mut().find(|(c, _)| *c == gene) {
                if entry.1 > 0 {
                    child[i] = gene;
                    filled[i] = true;
                    entry.1 -= 1;
                }
            }
        }
    }

    // 4. Fill gaps from P2 (Order preserving for remaining needs)
    let mut p2_idx = 0;
    for i in 0..len {
        if !filled[i] {
            // Find next available candidate in P2
            while p2_idx < len {
                let gene = p2[p2_idx];
                p2_idx += 1;

                if let Some(entry) = needed.iter_mut().find(|(c, _)| *c == gene) {
                    if entry.1 > 0 {
                        child[i] = gene;
                        filled[i] = true;
                        entry.1 -= 1;
                        break;
                    }
                }
            }
        }
    }

    // 5. Emergency Fallback (should logically never be hit if P1 and P2 are permutations)
    // If P2 runs out but we still have gaps (e.g. parents were different lengths or mismatched sets),
    // fill with KC_NO (0).
    for i in 0..len {
        if !filled[i] {
            child[i] = 0;
        }
    }

    child
}
