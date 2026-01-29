//! Formal verification harnesses for the KeyForge model types.
//! These proofs are executed using the Kani Rust Verifier.

#[cfg(kani)]
mod verification {
    use crate::Score;

    #[kani::proof]
    fn verify_score_from_f32_safety() {
        let f: f32 = kani::any();
        // The goal is to prove that calling from_f32 never panics
        // regardless of whether it returns Ok or Err.
        let _ = Score::from_f32(f);
    }

    #[kani::proof]
    fn verify_score_addition_saturation() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        let s1 = Score::from_scaled_i64(a);
        let s2 = Score::from_scaled_i64(b);
        let res = s1 + s2;
        
        // Manual verification of saturating property
        let expected = a.saturating_add(b);
        assert!(res.0 == expected);
    }
}
