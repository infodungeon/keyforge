use crate::supervisor::traits::AcceptanceCriteria;
use keyforge_model::constants::{ANNEALING_MIN_TEMP, SCORE_SCALE};
use rand::Rng;

#[derive(Debug)]
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
        // FIX: Use SCORE_SCALE instead of hardcoded 1,000,000.0
        let delta_f = delta as f32 / SCORE_SCALE;
        (-delta_f / temp).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_annealing_acceptance() {
        let mut criteria = CoolingAnnealing;
        let mut rng = StepRng::new(0, 0); // Always returns 0 (less than any probability)

        // 1. Improvement always accepted
        assert!(criteria.should_accept(-100, 1.0, &mut rng));

        // 2. High temp accepts degradation
        assert!(criteria.should_accept(100, 1000.0, &mut rng));

        // 3. Low temp rejects degradation
        let mut rng_high = StepRng::new(u64::MAX, 0); // Always returns 1.0
        assert!(!criteria.should_accept(1000000, 0.000001, &mut rng_high));
    }
}
