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

use crate::config::weights::constants::{
    DEFAULT_BONUS_INWARD_ROLL, DEFAULT_FINGER_PENALTY_SCALE_ARRAY, DEFAULT_LOADER_TRIGRAM_LIMIT,
    DEFAULT_PENALTY_REDIRECT, DEFAULT_PENALTY_SCISSOR, DEFAULT_PENALTY_SFB_BASE,
    DEFAULT_PENALTY_SFB_DIAGONAL, DEFAULT_PENALTY_SFB_LATERAL, DEFAULT_PENALTY_SFB_LATERAL_WEAK,
    DEFAULT_PENALTY_SFB_LONG, DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF,
    DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF, DEFAULT_TRIGRAM_COVERAGE, DEFAULT_WEIGHT_LATERAL_TRAVEL,
    DEFAULT_WEIGHT_VERTICAL_TRAVEL,
};
use crate::error::ForgeError;
use crate::types::Score;
use serde::{Deserialize, Serialize};

/// Raw representation of a Rubric for serialization (DTO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRubric {
    // --- Monograms ---
    /// Effort multipliers for each finger (0=Thumb, 4=Pinky).
    pub finger_effort: [Score; 5],
    /// Weight for lateral finger travel.
    pub travel_lat: Score,
    /// Weight for vertical finger travel.
    pub travel_vert: Score,

    // --- Bigrams (Same Finger) ---
    /// Base penalty for Same Finger Bigrams.
    pub sfb_base: Score,
    /// Penalty for lateral SFBs.
    pub sfb_lateral: Score,
    /// Penalty for lateral SFB on a weak finger.
    pub sfb_lateral_weak: Score,
    /// Penalty for diagonal SFBs.
    pub sfb_diagonal: Score,
    /// Penalty for long-distance SFBs.
    pub sfb_long: Score,
    /// Row difference threshold for "long" SFBs.
    pub threshold_sfb_long_row_diff: i8,

    // --- Bigrams (Adjacent Finger) ---
    /// Penalty for scissor (adjacent finger stretch) movements.
    pub penalty_scissor: Score,
    /// Row difference threshold for scissors.
    pub threshold_scissor_row_diff: i8,

    // --- Flow & Trigrams ---
    /// Penalty for redirects (direction changes).
    pub redirect: Score,
    /// Bonus for inward rolls.
    pub roll_bonus: Score,
    /// Bonus for outward rolls.
    pub roll_out_bonus: Score,
    /// Required trigram coverage ratio.
    pub trigram_coverage: Score,
    /// Maximum number of trigrams to consider.
    pub trigram_limit: usize,
}

impl Default for RawRubric {
    fn default() -> Self {
        Self {
            finger_effort: DEFAULT_FINGER_PENALTY_SCALE_ARRAY,
            travel_lat: DEFAULT_WEIGHT_LATERAL_TRAVEL,
            travel_vert: DEFAULT_WEIGHT_VERTICAL_TRAVEL,
            sfb_base: DEFAULT_PENALTY_SFB_BASE,
            sfb_lateral: DEFAULT_PENALTY_SFB_LATERAL,
            sfb_lateral_weak: DEFAULT_PENALTY_SFB_LATERAL_WEAK,
            sfb_diagonal: DEFAULT_PENALTY_SFB_DIAGONAL,
            sfb_long: DEFAULT_PENALTY_SFB_LONG,
            threshold_sfb_long_row_diff: DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF,
            penalty_scissor: DEFAULT_PENALTY_SCISSOR,
            threshold_scissor_row_diff: DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF,
            redirect: DEFAULT_PENALTY_REDIRECT,
            roll_bonus: DEFAULT_BONUS_INWARD_ROLL,
            roll_out_bonus: Score::from_scaled_i64(15_000_000), // 15.0
            trigram_coverage: DEFAULT_TRIGRAM_COVERAGE,
            trigram_limit: DEFAULT_LOADER_TRIGRAM_LIMIT,
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
    pub fn finger_effort(mut self, effort: [i64; 5]) -> Self {
        let sc = |v: i64| Score::from_scaled_i64(v);
        self.raw.finger_effort = [
            sc(effort[0]),
            sc(effort[1]),
            sc(effort[2]),
            sc(effort[3]),
            sc(effort[4]),
        ];
        self
    }

    /// Sets the lateral travel weight.
    #[must_use]
    pub fn travel_lat(mut self, travel: i64) -> Self {
        self.raw.travel_lat = Score::from_scaled_i64(travel);
        self
    }

    /// Sets the vertical travel weight.
    #[must_use]
    pub fn travel_vert(mut self, travel: i64) -> Self {
        self.raw.travel_vert = Score::from_scaled_i64(travel);
        self
    }

    /// Sets the base SFB penalty.
    #[must_use]
    pub fn sfb_base(mut self, penalty: i64) -> Self {
        self.raw.sfb_base = Score::from_scaled_i64(penalty);
        self
    }

    /// Sets the lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral(mut self, penalty: i64) -> Self {
        self.raw.sfb_lateral = Score::from_scaled_i64(penalty);
        self
    }

    /// Sets the weak-finger lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral_weak(mut self, penalty: i64) -> Self {
        self.raw.sfb_lateral_weak = Score::from_scaled_i64(penalty);
        self
    }

    /// Sets the diagonal SFB penalty.
    #[must_use]
    pub fn sfb_diagonal(mut self, penalty: i64) -> Self {
        self.raw.sfb_diagonal = Score::from_scaled_i64(penalty);
        self
    }

    /// Sets the long-reach SFB penalty.
    #[must_use]
    pub fn sfb_long(mut self, penalty: i64) -> Self {
        self.raw.sfb_long = Score::from_scaled_i64(penalty);
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
    pub fn penalty_scissor(mut self, penalty: i64) -> Self {
        self.raw.penalty_scissor = Score::from_scaled_i64(penalty);
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
    pub fn redirect(mut self, penalty: i64) -> Self {
        self.raw.redirect = Score::from_scaled_i64(penalty);
        self
    }

    /// Sets the inward roll bonus.
    #[must_use]
    pub fn roll_bonus(mut self, bonus: i64) -> Self {
        self.raw.roll_bonus = Score::from_scaled_i64(bonus);
        self
    }

    /// Sets the outward roll bonus.
    #[must_use]
    pub fn roll_out_bonus(mut self, bonus: i64) -> Self {
        self.raw.roll_out_bonus = Score::from_scaled_i64(bonus);
        self
    }

    /// Sets the trigram coverage requirement.
    #[must_use]
    pub fn trigram_coverage(mut self, coverage: i64) -> Self {
        self.raw.trigram_coverage = Score::from_scaled_i64(coverage);
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
    pub fn finger_effort(&self) -> &[Score; 5] {
        &self.inner.finger_effort
    }
    /// Returns the lateral travel weight.
    #[must_use]
    pub fn travel_lat(&self) -> Score {
        self.inner.travel_lat
    }

    /// Returns the vertical travel weight.
    #[must_use]
    pub fn travel_vert(&self) -> Score {
        self.inner.travel_vert
    }

    /// Returns the base SFB penalty.
    #[must_use]
    pub fn sfb_base(&self) -> Score {
        self.inner.sfb_base
    }

    /// Returns the lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral(&self) -> Score {
        self.inner.sfb_lateral
    }

    /// Returns the weak-finger lateral SFB penalty.
    #[must_use]
    pub fn sfb_lateral_weak(&self) -> Score {
        self.inner.sfb_lateral_weak
    }

    /// Returns the diagonal SFB penalty.
    #[must_use]
    pub fn sfb_diagonal(&self) -> Score {
        self.inner.sfb_diagonal
    }

    /// Returns the long-reach SFB penalty.
    #[must_use]
    pub fn sfb_long(&self) -> Score {
        self.inner.sfb_long
    }

    /// Returns the row difference threshold for long SFBs.
    #[must_use]
    pub fn threshold_sfb_long_row_diff(&self) -> i8 {
        self.inner.threshold_sfb_long_row_diff
    }

    /// Returns the scissor penalty.
    #[must_use]
    pub fn penalty_scissor(&self) -> Score {
        self.inner.penalty_scissor
    }

    /// Returns the row difference threshold for scissors.
    #[must_use]
    pub fn threshold_scissor_row_diff(&self) -> i8 {
        self.inner.threshold_scissor_row_diff
    }

    /// Returns the redirect penalty.
    #[must_use]
    pub fn redirect(&self) -> Score {
        self.inner.redirect
    }

    /// Returns the inward roll bonus.
    #[must_use]
    pub fn roll_bonus(&self) -> Score {
        self.inner.roll_bonus
    }

    /// Returns the outward roll bonus.
    #[must_use]
    pub fn roll_out_bonus(&self) -> Score {
        self.inner.roll_out_bonus
    }

    /// Returns the trigram coverage requirement.
    #[must_use]
    pub fn trigram_coverage(&self) -> Score {
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
        if self.inner.trigram_coverage < Score::ZERO
            || self.inner.trigram_coverage > Score::from_scaled_i64(1_000_000)
        {
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
        Ok(())
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_rubric_lifecycle() -> anyhow::Result<()> {
        // 1. Default Construction
        let r = Rubric::default();

        // Check key defaults to ensure physics engine gets sensible start values
        assert!(r.sfb_base() > Score::ZERO);
        assert!(r.travel_lat() > Score::ZERO);
        assert!(r.travel_vert() > Score::ZERO);
        assert_eq!(r.finger_effort().len(), 5);

        // 2. Serialization Round-trip
        let json = serde_json::to_string(&r)?;
        let recovered: Rubric = serde_json::from_str(&json)?;

        // 3. Verification
        assert_eq!(r.sfb_base(), recovered.sfb_base());
        assert_eq!(r.finger_effort(), recovered.finger_effort());
        Ok(())
    }

    #[test]
    fn test_rubric_modification() -> anyhow::Result<()> {
        let mut raw = RawRubric::default();
        let sc = |v: i64| Score::from_scaled_i64(v);
        raw.sfb_base = sc(1_000_000_000); // 1000.0
        raw.finger_effort[4] = sc(5_000_000); // Pinky penalty 5.0

        let r = Rubric::from(raw);
        assert_eq!(r.sfb_base(), sc(1_000_000_000));
        assert_eq!(r.finger_effort()[4], sc(5_000_000));
        Ok(())
    }

    #[test]
    fn test_rubric_validation() -> anyhow::Result<()> {
        let mut raw = RawRubric::default();
        let sc = |v: i64| Score::from_scaled_i64(v);
        assert!(Rubric::from(raw.clone()).validate().is_ok());

        // Coverage bounds
        raw.trigram_coverage = Score::from_scaled_i64(2_000_000); // 2.0
        assert!(Rubric::from(raw.clone()).validate().is_err());
        raw.trigram_coverage = Score::from_scaled_i64(-100_000); // -0.1
        assert!(Rubric::from(raw.clone()).validate().is_err());

        // Reset to valid
        raw.trigram_coverage = sc(990_000); // 0.99

        // Limits
        raw.trigram_limit = 0;
        assert!(Rubric::from(raw.clone()).validate().is_err());
        raw.trigram_limit = 100;

        // Weights
        raw.sfb_base = Score::from_scaled_i64(-10_000_000); // Negative penalty
        assert!(Rubric::from(raw.clone()).validate().is_err());

        raw.sfb_base = sc(400_000_000); // 400.0
        raw.sfb_lateral = Score::from_scaled_i64(-1_000_000);
        assert!(Rubric::from(raw).validate().is_err());
        Ok(())
    }
}
