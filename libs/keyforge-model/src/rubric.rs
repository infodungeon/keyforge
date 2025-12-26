use serde::{Deserialize, Serialize};

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
