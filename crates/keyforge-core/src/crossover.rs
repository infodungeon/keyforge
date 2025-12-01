use fastrand::Rng;
use std::collections::HashMap;

/// Performs Uniform Order Crossover (UOX) on two layouts.
///
/// This ensures the child layout is a valid permutation of the parents
/// (conserving the exact count of every character, including 0/nulls).
pub fn crossover_uniform(p1: &[u8], p2: &[u8], rng: &mut Rng) -> Vec<u8> {
    let len = p1.len();
    assert_eq!(len, p2.len(), "Parents must have same length");

    let mut child = vec![0u8; len];
    let mut filled = vec![false; len];

    // Track the 'budget' of characters available to inherit.
    // This handles duplicates (like multiple 0s for empty keys) correctly.
    let mut available_counts: HashMap<u8, usize> = HashMap::new();
    for &b in p1 {
        *available_counts.entry(b).or_default() += 1;
    }

    // 1. Inherit from Parent 1 based on random mask
    // (Roughly 50% chance to keep position from P1)
    for i in 0..len {
        if rng.bool() {
            let gene = p1[i];
            child[i] = gene;
            filled[i] = true;
            
            // Decrement available count
            if let Some(count) = available_counts.get_mut(&gene) {
                *count -= 1;
            }
        }
    }

    // 2. Fill gaps from Parent 2 (Preserving relative order)
    let mut p2_idx = 0;
    for i in 0..len {
        if !filled[i] {
            // Find the next gene in P2 that we still 'need'
            while p2_idx < len {
                let gene = p2[p2_idx];
                p2_idx += 1;

                if let Some(count) = available_counts.get_mut(&gene) {
                    if *count > 0 {
                        // Found a valid candidate
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