use serde::{Deserialize, Serialize};
use crate::error::ForgeError;

/// Configuration for the Physics Engine.
/// Defines "What is expensive?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    // SFB Penalties
    pub sfb_base: f32,
    pub sfb_lateral: f32,

    // Geometry Weights
    pub travel_lat: f32,
    pub travel_vert: f32,

    // Finger Weights (Thumb..Pinky)
    pub finger_effort: [f32; 5],

    // Flow
    pub redirect: f32,
    pub roll_bonus: f32,
    pub trigram_coverage: f32,
    pub trigram_limit: usize,
}

impl Default for Rubric {
    fn default() -> Self {
        Self {
            sfb_base: 400.0,
            sfb_lateral: 65.0,
            travel_lat: 3.5,
            travel_vert: 1.0,
            finger_effort: [0.0, 1.0, 1.1, 1.3, 1.6],
            redirect: 65.0,
            roll_bonus: 35.0,
            trigram_coverage: 0.99,
            trigram_limit: 50_000,
        }
    }
}

impl Rubric {
    pub fn validate(&self) -> Result<(), ForgeError> {
        if self.trigram_coverage < 0.0 || self.trigram_coverage > 1.0 {
            return Err(ForgeError::InvalidData(format!(
                "Trigram coverage must be between 0.0 and 1.0, found {}",
                self.trigram_coverage
            )));
        }
        if self.trigram_limit == 0 {
            return Err(ForgeError::InvalidData(
                "Trigram limit must be greater than 0".into(),
            ));
        }
        // Basic sanity checks for weights (optional, but good practice)
        if self.sfb_base < 0.0 || self.sfb_lateral < 0.0 {
             return Err(ForgeError::InvalidData(
                "SFB penalties cannot be negative".into(),
            ));
        }
        Ok(())
    }
}
