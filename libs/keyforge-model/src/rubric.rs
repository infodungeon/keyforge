// libs/keyforge-model/src/rubric.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! Scoring configuration and weights.
//!
//! A `Rubric` defines the cost parameters used by the physics engine 
//! to evaluate the efficiency of a layout.

use serde::{Deserialize, Serialize};
use crate::error::ForgeError;
use crate::constants::{EFFORT_THUMB, EFFORT_INDEX, EFFORT_MIDDLE, EFFORT_RING, EFFORT_PINKY};

/// Configuration for the Physics Engine.
/// Defines "What is expensive?" by assigning weights to physical movements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    // SFB Penalties
    /// Base penalty for Same Finger Bigrams.
    pub sfb_base: f32,
    /// Penalty for lateral SFBs.
    pub sfb_lateral: f32,

    // Geometry Weights
    /// Weight for lateral finger travel.
    pub travel_lat: f32,
    /// Weight for vertical finger travel.
    pub travel_vert: f32,

    // Finger Weights (Thumb..Pinky)
    /// Effort multipliers for each finger (0=Thumb, 4=Pinky).
    pub finger_effort: [f32; 5],

    // Flow
    /// Penalty for redirects (direction changes).
    pub redirect: f32,
    /// Bonus for inward rolls.
    pub roll_bonus: f32,
    /// Required trigram coverage ratio.
    pub trigram_coverage: f32,
    /// Maximum number of trigrams to consider.
    pub trigram_limit: usize,
}

impl Default for Rubric {
    fn default() -> Self {
        Self {
            sfb_base: 400.0,
            sfb_lateral: 65.0,
            travel_lat: 3.5,
            travel_vert: 1.0,
            finger_effort: [EFFORT_THUMB, EFFORT_INDEX, EFFORT_MIDDLE, EFFORT_RING, EFFORT_PINKY],
            redirect: 65.0,
            roll_bonus: 35.0,
            trigram_coverage: 0.99,
            trigram_limit: 50_000,
        }
    }
}

impl Rubric {
    /// Validates the rubric configuration.
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
