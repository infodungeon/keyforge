use fastrand::Rng;
use keyforge_core::optimizer::crossover::crossover_uniform;

#[test]
fn test_crossover_validity() {
    let mut rng = Rng::with_seed(42);

    // Parent 1: "ABCDE" (mapped to bytes)
    let p1 = vec![1, 2, 3, 4, 5];
    // Parent 2: "EDCBA"
    let p2 = vec![5, 4, 3, 2, 1];

    // UPDATED: Pass empty pins
    let no_pins = vec![None; 5];
    let child = crossover_uniform(&p1, &p2, &no_pins, &mut rng);

    // 1. Length check
    assert_eq!(child.len(), 5);

    // 2. Conservation check (Must contain 1,2,3,4,5 exactly once)
    let mut p1_sorted = p1.clone();
    p1_sorted.sort();
    let mut child_sorted = child.clone();
    child_sorted.sort();

    assert_eq!(p1_sorted, child_sorted, "Child lost or duplicated genes!");

    // 3. Mixing check (Unlikely to match parents exactly with seed 42)
    assert_ne!(child, p1);
    assert_ne!(child, p2);
}

#[test]
fn test_crossover_with_duplicates() {
    let mut rng = Rng::with_seed(100);

    // Layout with empty keys (0) and duplicates (if any)
    // A, B, 0, 0, C
    let p1 = vec![65, 66, 0, 0, 67];
    // 0, C, B, A, 0
    let p2 = vec![0, 67, 66, 65, 0];

    // UPDATED: Pass empty pins
    let no_pins = vec![None; 5];

    for _ in 0..100 {
        let child = crossover_uniform(&p1, &p2, &no_pins, &mut rng);

        let zeros = child.iter().filter(|&&x| x == 0).count();
        let a = child.iter().filter(|&&x| x == 65).count();
        let b = child.iter().filter(|&&x| x == 66).count();
        let c = child.iter().filter(|&&x| x == 67).count();

        assert_eq!(zeros, 2, "Must preserve exactly two 0s");
        assert_eq!(a, 1);
        assert_eq!(b, 1);
        assert_eq!(c, 1);
    }
}

#[test]
fn test_crossover_respects_pins() {
    let mut rng = Rng::with_seed(42);

    // Parent 1: A B C D E
    let p1 = vec![65, 66, 67, 68, 69];
    // Parent 2: E D C B A
    let p2 = vec![69, 68, 67, 66, 65];

    // Pin 'A' (65) to index 0, and 'B' (66) to index 4
    let mut pinned = vec![None; 5];
    pinned[0] = Some(65);
    pinned[4] = Some(66);

    // Run multiple iterations to ensure RNG doesn't accidentally succeed
    for _ in 0..100 {
        let child = crossover_uniform(&p1, &p2, &pinned, &mut rng);

        // 1. Check Constraint: Pins must be respected
        assert_eq!(child[0], 65, "Index 0 must be A");
        assert_eq!(child[4], 66, "Index 4 must be B");

        // 2. Check Conservation: All elements must be present exactly once
        let mut sorted = child.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            vec![65, 66, 67, 68, 69],
            "Child lost or duplicated genes!"
        );
    }
}
