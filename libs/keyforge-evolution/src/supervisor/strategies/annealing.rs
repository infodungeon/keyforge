use crate::supervisor::traits::AcceptanceCriteria;
use keyforge_model::constants::{ANNEALING_MIN_TEMP, SCORE_SCALE};
use rand::Rng;

#[derive(Debug, Clone, Copy, Default)]
pub struct CoolingAnnealing;

impl AcceptanceCriteria for CoolingAnnealing {
    fn should_accept(&mut self, delta: i64, temperature: f32, rng: &mut impl Rng) -> bool {
        if delta <= 0 {
            return true;
        }

        if temperature < ANNEALING_MIN_TEMP {
            return false;
        }

        // INVARIANT: kani::assume(temperature > 0.0);
        let probability = Self::get_acceptance_prob(delta, temperature);
        rng.random::<f32>() < probability
    }
}

impl CoolingAnnealing {
    #[allow(clippy::cast_precision_loss)]
    fn get_acceptance_prob(delta: i64, temp: f32) -> f32 {
        let delta_f = delta as f32 / SCORE_SCALE;
        (-delta_f / temp).exp()
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use rand::RngCore;

    #[derive(Debug)]
    struct MockRng(u64);
    impl RngCore for MockRng {
        fn next_u32(&mut self) -> u32 {
            self.0 as u32
        }
        fn next_u64(&mut self) -> u64 {
            self.0
        }
        fn fill_bytes(&mut self, _dest: &mut [u8]) {}
    }

    #[test]
    fn test_annealing_acceptance() {
        let mut criteria = CoolingAnnealing;

        // 1. Improvement always accepted
        let mut rng = MockRng(u64::MAX); // High value (1.0)
        assert!(criteria.should_accept(-100, 1.0, &mut rng));

        // 2. High temp accepts degradation
        let mut rng_low = MockRng(0); // Low value (0.0)
        assert!(criteria.should_accept(100, 1000.0, &mut rng_low));

        // 3. Low temp rejects degradation
        let mut rng_high = MockRng(u64::MAX); // High value (1.0)
        assert!(!criteria.should_accept(1_000_000, 0.000_001, &mut rng_high));

        // 4. Below minimum temp rejects everything positive
        assert!(!criteria.should_accept(1, 1e-12, &mut rng_low));
    }
}
