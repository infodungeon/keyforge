use fastrand::Rng;

// Optimized Uniform Crossover (UOX)
pub fn crossover_uniform(
    p1: &[u16],
    p2: &[u16],
    pinned: &[Option<u16>],
    rng: &mut Rng,
) -> Vec<u16> {
    let len = p1.len();
    let mut child = vec![0u16; len];
    let mut filled = vec![false; len];

    // 1. Apply Pins
    for (i, &pin) in pinned.iter().enumerate() {
        if i < len {
            if let Some(val) = pin {
                child[i] = val;
                filled[i] = true;
            }
        }
    }

    // 2. Build Frequency Table
    let mut needed: Vec<(u16, u8)> = Vec::with_capacity(len);

    for &code in p1 {
        if let Some(entry) = needed.iter_mut().find(|(c, _)| *c == code) {
            entry.1 += 1;
        } else {
            needed.push((code, 1));
        }
    }

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

    // 3. Inherit from P1
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

    // 4. Fill gaps from P2
    let mut p2_idx = 0;
    for i in 0..len {
        if !filled[i] {
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

    // 5. Emergency Fallback
    for i in 0..len {
        if !filled[i] {
            child[i] = 0;
        }
    }

    child
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn get_sorted(vec: &[u16]) -> Vec<u16> {
        let mut v = vec.to_vec();
        v.sort();
        v
    }

    #[test]
    fn test_conservation_basic() {
        let mut rng = fastrand::Rng::with_seed(42);
        let p1 = vec![1, 2, 3, 4, 5];
        let p2 = vec![5, 4, 3, 2, 1];
        let pins = vec![None; 5];

        let child = crossover_uniform(&p1, &p2, &pins, &mut rng);
        assert_eq!(child.len(), 5);
        assert_eq!(
            get_sorted(&child),
            vec![1, 2, 3, 4, 5],
            "Mass not conserved!"
        );
    }

    #[test]
    fn test_pin_preservation() {
        let mut rng = fastrand::Rng::with_seed(42);
        let p1 = vec![1, 2, 3, 4];
        let p2 = vec![4, 3, 2, 1];
        let mut pins = vec![None; 4];
        pins[0] = Some(1);

        let child = crossover_uniform(&p1, &p2, &pins, &mut rng);
        assert_eq!(child[0], 1, "Pin ignored");
        assert_eq!(get_sorted(&child), vec![1, 2, 3, 4], "Mass lost");
    }

    proptest! {
        #[test]
        fn prop_conservation_of_mass(
            seed in any::<u64>()
        ) {
            let mut rng = fastrand::Rng::with_seed(seed);
            let p1 = vec![10, 20, 30, 40, 50, 60, 70, 80];
            let mut p2 = p1.clone();
            p2.reverse();

            let pins = vec![None; 8];
            let child = crossover_uniform(&p1, &p2, &pins, &mut rng);

            let mut s_p1 = p1.clone(); s_p1.sort();
            let mut s_c = child.clone(); s_c.sort();

            prop_assert_eq!(s_p1, s_c, "Child genes differ from parent genes");
        }
    }
}
