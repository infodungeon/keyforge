use crate::supervisor::traits::AcceptanceCriteria;
use keyforge_model::constants::{ANNEALING_MIN_TEMP, SCORE_SCALE};
use rand::Rng;

pub struct CoolingAnnealing;

impl AcceptanceCriteria for CoolingAnnealing {
    fn should_accept(&mut self, delta: i64, temperature: f32, rng: &mut impl Rng) -> bool {
        if delta <= 0 {
            return true;
        }

        if temperature < ANNEALING_MIN_TEMP {
            return false;
        }

        // FIX: Use SCORE_SCALE instead of hardcoded 1,000,000.0
        let delta_f = delta as f32 / SCORE_SCALE;
        // INVARIANT: kani::assume(temperature > 0.0);
        let probability = (-delta_f / temperature).exp();
        rng.random::<f32>() < probability
    }
}
