use keyforge_core::optimizer::crossover::crossover_uniform;
use fastrand::Rng;

#[test]
fn test_crossover_validity() {
    let mut rng = Rng::with_seed(42);

    // Parent 1: "ABCDE" (mapped to bytes)
    let p1 = vec![1, 2, 3, 4, 5];
    // Parent 2: "EDCBA"
    let p2 = vec![5, 4, 3, 2, 1];

    let child = crossover_uniform(&p1, &p2, &mut rng);

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

    for _ in 0..100 {
        let child = crossover_uniform(&p1, &p2, &mut rng);
        
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