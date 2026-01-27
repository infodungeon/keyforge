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

use crate::error::ForgeError;
use serde::{Deserialize, Serialize};

/// Raw representation of a Rubric for serialization (DTO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRubric {
    // --- Monograms ---
    /// Effort multipliers for each finger (0=Thumb, 4=Pinky).
    pub finger_effort: [f32; 5],
    /// Weight for lateral finger travel.
    pub travel_lat: f32,
    /// Weight for vertical finger travel.
    pub travel_vert: f32,

    // --- Bigrams (Same Finger) ---
    /// Base penalty for Same Finger Bigrams.
    pub sfb_base: f32,
    /// Penalty for lateral SFBs.
    pub sfb_lateral: f32,
    /// Penalty for lateral SFB on a weak finger.
    pub sfb_lateral_weak: f32,
    /// Penalty for diagonal SFBs.
    pub sfb_diagonal: f32,
    /// Penalty for long-distance SFBs.
    pub sfb_long: f32,
    /// Row difference threshold for "long" SFBs.
    pub threshold_sfb_long_row_diff: i8,

    // --- Bigrams (Adjacent Finger) ---
    /// Penalty for scissor (adjacent finger stretch) movements.
    pub penalty_scissor: f32,
    /// Row difference threshold for scissors.
    pub threshold_scissor_row_diff: i8,

    // --- Flow & Trigrams ---
    /// Penalty for redirects (direction changes).
    pub redirect: f32,
    /// Bonus for inward rolls.
    pub roll_bonus: f32,
    /// Bonus for outward rolls.
    pub roll_out_bonus: f32,
    /// Required trigram coverage ratio.
    pub trigram_coverage: f32,
    /// Maximum number of trigrams to consider.
    pub trigram_limit: usize,
}

impl Default for RawRubric {
    fn default() -> Self {
        Self {
            finger_effort: [1.0, 1.0, 1.1, 1.3, 1.6],
            travel_lat: 3.5,
            travel_vert: 1.0,
            sfb_base: 400.0,
            sfb_lateral: 65.0,
            sfb_lateral_weak: 160.0,
            sfb_diagonal: 240.0,
            sfb_long: 280.0,
            threshold_sfb_long_row_diff: 2,
            penalty_scissor: 25.0,
            threshold_scissor_row_diff: 2,
            redirect: 65.0,
            roll_bonus: 35.0,
            roll_out_bonus: 15.0,
            trigram_coverage: 0.99,
            trigram_limit: 50_000,
        }
    }
}

/// Validated Scoring configuration (Domain Model).
/// Defines "What is expensive?" by assigning weights to physical movements.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(from = "RawRubric", into = "RawRubric")]
pub struct Rubric {
    inner: RawRubric,
}

impl From<RawRubric> for Rubric {
    fn from(raw: RawRubric) -> Self {
        // Note: In production, we might want to return Result,
        // but since this is used in #[serde(from)], we fallback to validation at usage or panic-free defaults.
        Self { inner: raw }
    }
}

impl From<Rubric> for RawRubric {
    fn from(r: Rubric) -> Self {
        r.inner
    }
}

/// Builder for `Rubric`.
#[derive(Debug, Clone, Default)]
pub struct RubricBuilder {
    raw: RawRubric,
}

impl RubricBuilder {
    /// Consumes the builder and returns a `Rubric`.
    #[must_use]
    pub fn build(self) -> Rubric {
        Rubric::from(self.raw)
    }

    /// Sets the per-finger effort weights.
    #[must_use]
    pub fn finger_effort(mut self, effort: [f32; 5]) -> Self {
        self.raw.finger_effort = effort;
        self
    }

    /// Sets the lateral travel weight.
    #[must_use]
    pub fn travel_lat(mut self, travel: f32) -> Self {
        self.raw.travel_lat = travel;
        self
    }

    /// Sets the vertical travel weight.
    #[must_use]
    pub fn travel_vert(mut self, travel: f32) -> Self {
        self.raw.travel_vert = travel;
        self
    }

    /// Sets the base SFB penalty.
    #[must_use]
    pub fn sfb_base(mut self, penalty: f32) -> Self {
        self.raw.sfb_base = penalty;
        self
    }

    /// Sets the lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral(mut self, penalty: f32) -> Self {
        self.raw.sfb_lateral = penalty;
        self
    }

    /// Sets the weak-finger lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral_weak(mut self, penalty: f32) -> Self {
        self.raw.sfb_lateral_weak = penalty;
        self
    }

    /// Sets the diagonal SFB penalty.
    #[must_use]
    pub fn sfb_diagonal(mut self, penalty: f32) -> Self {
        self.raw.sfb_diagonal = penalty;
        self
    }

    /// Sets the long-reach SFB penalty.
    #[must_use]
    pub fn sfb_long(mut self, penalty: f32) -> Self {
        self.raw.sfb_long = penalty;
        self
    }

    /// Sets the row difference threshold for long SFBs.
    #[must_use]
    pub fn threshold_sfb_long_row_diff(mut self, threshold: i8) -> Self {
        self.raw.threshold_sfb_long_row_diff = threshold;
        self
    }

    /// Sets the scissor penalty.
    #[must_use]
    pub fn penalty_scissor(mut self, penalty: f32) -> Self {
        self.raw.penalty_scissor = penalty;
        self
    }

    /// Sets the row difference threshold for scissors.
    #[must_use]
    pub fn threshold_scissor_row_diff(mut self, threshold: i8) -> Self {
        self.raw.threshold_scissor_row_diff = threshold;
        self
    }

    /// Sets the redirect penalty.
    #[must_use]
    pub fn redirect(mut self, penalty: f32) -> Self {
        self.raw.redirect = penalty;
        self
    }

    /// Sets the inward roll bonus.
    #[must_use]
    pub fn roll_bonus(mut self, bonus: f32) -> Self {
        self.raw.roll_bonus = bonus;
        self
    }

    /// Sets the outward roll bonus.
    #[must_use]
    pub fn roll_out_bonus(mut self, bonus: f32) -> Self {
        self.raw.roll_out_bonus = bonus;
        self
    }

    /// Sets the trigram coverage requirement.
    #[must_use]
    pub fn trigram_coverage(mut self, coverage: f32) -> Self {
        self.raw.trigram_coverage = coverage;
        self
    }

    /// Sets the maximum number of top trigrams to consider.
    #[must_use]
    pub fn trigram_limit(mut self, limit: usize) -> Self {
        self.raw.trigram_limit = limit;
        self
    }
}

impl Rubric {
    /// Creates a new Rubric builder.
    #[must_use]
    pub fn builder() -> RubricBuilder {
        RubricBuilder::default()
    }

    /// Returns the per-finger effort weights.
    #[must_use]
    pub fn finger_effort(&self) -> &[f32; 5] {
        &self.inner.finger_effort
    }
    /// Returns the lateral travel weight.
    #[must_use]
    pub fn travel_lat(&self) -> f32 {
        self.inner.travel_lat
    }

    /// Returns the vertical travel weight.
    #[must_use]
    pub fn travel_vert(&self) -> f32 {
        self.inner.travel_vert
    }

    /// Returns the base SFB penalty.
    #[must_use]
    pub fn sfb_base(&self) -> f32 {
        self.inner.sfb_base
    }

    /// Returns the lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral(&self) -> f32 {
        self.inner.sfb_lateral
    }

    /// Returns the weak-finger lateral SFB penalty.
    /// Returns the lateral SFB penalty on a weak finger.
    #[must_use]
    pub fn sfb_lateral_weak(&self) -> f32 {
        self.inner.sfb_lateral_weak
    }

    /// Returns the diagonal SFB penalty.
    #[must_use]
    pub fn sfb_diagonal(&self) -> f32 {
        self.inner.sfb_diagonal
    }

    /// Returns the long-reach SFB penalty.
    #[must_use]
    pub fn sfb_long(&self) -> f32 {
        self.inner.sfb_long
    }

    /// Returns the row difference threshold for long SFBs.
    #[must_use]
    pub fn threshold_sfb_long_row_diff(&self) -> i8 {
        self.inner.threshold_sfb_long_row_diff
    }

    /// Returns the scissor penalty.
    #[must_use]
    pub fn penalty_scissor(&self) -> f32 {
        self.inner.penalty_scissor
    }

    /// Returns the row difference threshold for scissors.
    #[must_use]
    pub fn threshold_scissor_row_diff(&self) -> i8 {
        self.inner.threshold_scissor_row_diff
    }

    /// Returns the redirect penalty.
    #[must_use]
    pub fn redirect(&self) -> f32 {
        self.inner.redirect
    }

    /// Returns the inward roll bonus.
    #[must_use]
    pub fn roll_bonus(&self) -> f32 {
        self.inner.roll_bonus
    }

    /// Returns the outward roll bonus.
    #[must_use]
    pub fn roll_out_bonus(&self) -> f32 {
        self.inner.roll_out_bonus
    }

    /// Returns the trigram coverage requirement.
    #[must_use]
    pub fn trigram_coverage(&self) -> f32 {
        self.inner.trigram_coverage
    }

    /// Returns the maximum number of top trigrams to consider.
    #[must_use]
    pub fn trigram_limit(&self) -> usize {
        self.inner.trigram_limit
    }

    /// Validates the rubric configuration.
    ///
    /// # Errors
    ///
    /// Returns a `ForgeError` if the trigram coverage is out of range, or if
    /// trigram limits/penalties are invalid.
    pub fn validate(&self) -> Result<(), ForgeError> {
        if self.inner.trigram_coverage < 0.0 || self.inner.trigram_coverage > 1.0 {
            return Err(ForgeError::InvalidData(format!(
                "Trigram coverage must be between 0.0 and 1.0, found {}",
                self.inner.trigram_coverage
            )));
        }
        if self.inner.trigram_limit == 0 {
            return Err(ForgeError::InvalidData(
                "Trigram limit must be greater than 0".into(),
            ));
        }
        if self.inner.sfb_base < 0.0 || self.inner.sfb_lateral < 0.0 {
            return Err(ForgeError::InvalidData(
                "SFB penalties cannot be negative".into(),
            ));
        }
        Ok(())
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_rubric_lifecycle() {
        // 1. Default Construction
        let r = Rubric::default();

        // Check key defaults to ensure physics engine gets sensible start values
        assert!(r.sfb_base() > 0.0);
        assert!(r.travel_lat() > 0.0);
        assert!(r.travel_vert() > 0.0);
        assert_eq!(r.finger_effort().len(), 5);

        // 2. Serialization Round-trip
        let json = serde_json::to_string(&r).expect("Failed to serialize Rubric");
        let recovered: Rubric = serde_json::from_str(&json).expect("Failed to deserialize Rubric");

        // 3. Verification
        assert_eq!(r.sfb_base(), recovered.sfb_base());
        assert_eq!(r.finger_effort(), recovered.finger_effort());
    }

    #[test]
    fn test_rubric_modification() {
        let mut raw = RawRubric::default();
        raw.sfb_base = 1000.0;
        raw.finger_effort[4] = 5.0; // Pinky penalty

        let r = Rubric::from(raw);
        assert_eq!(r.sfb_base(), 1000.0);
        assert_eq!(r.finger_effort()[4], 5.0);
    }

    #[test]
    fn test_rubric_validation() {
        let mut raw = RawRubric::default();
        assert!(Rubric::from(raw.clone()).validate().is_ok());

        // Coverage bounds
        raw.trigram_coverage = 1.5; // > 1.0
        assert!(Rubric::from(raw.clone()).validate().is_err());
        raw.trigram_coverage = -0.1;
        assert!(Rubric::from(raw.clone()).validate().is_err());

        // Reset to valid
        raw.trigram_coverage = 0.99;

        // Limits
        raw.trigram_limit = 0;
        assert!(Rubric::from(raw.clone()).validate().is_err());
        raw.trigram_limit = 100;

        // Weights
        raw.sfb_base = -10.0; // Negative penalty
        assert!(Rubric::from(raw.clone()).validate().is_err());

        raw.sfb_base = 400.0;
        raw.sfb_lateral = -1.0;
        assert!(Rubric::from(raw).validate().is_err());
    }
}
