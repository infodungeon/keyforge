// libs/keyforge-model/src/types/scoring.rs

use crate::constants::SCORE_SCALE;
use crate::types::FixedPointMath;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};
use utoipa::ToSchema;

/// Represents a biomechanical effort score in deterministic fixed-point units.
///
/// Scaling: 1,000,000 units = 1.0 Effort Point.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Score(i64);

impl FixedPointMath for Score {
    type Raw = i64;
    fn raw(self) -> Self::Raw {
        self.0
    }
    fn from_raw(val: Self::Raw) -> Self {
        Self(val)
    }
    fn scale() -> f64 {
        f64::from(SCORE_SCALE)
    }
}

impl Score {
    /// Maximum possible score.
    pub const MAX: Score = Score::from_scaled_i64(i64::MAX);
    /// Minimum possible score.
    pub const MIN: Score = Score::from_scaled_i64(i64::MIN);
    /// Zero score.
    pub const ZERO: Score = Score::from_scaled_i64(0);
    /// Sentinel value for unreachable or uninitialized costs.
    pub const INFINITY_SENTINEL: Score = Score::from_scaled_i64(i64::MAX);

    /// Returns the raw `i64` value.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// Creates a Score from a float value, applying scaling.
    ///
    /// # Errors
    /// Returns an error if the resulting value overflows `i64`.
    pub fn from_f32(val: f32) -> Result<Self, String> {
        if val.is_nan() {
            return Err("Cannot create Score from NaN".to_string());
        }
        let scaled = f64::from(val) * f64::from(SCORE_SCALE);
        if scaled > i64::MAX as f64 || scaled < i64::MIN as f64 {
            return Err(format!(
                "Score overflow: {val} * {SCORE_SCALE} exceeds i64 range"
            ));
        }
        #[allow(clippy::cast_possible_truncation)]
        Ok(Score::from_scaled_i64(scaled as i64))
    }

    /// Creates a Score from a raw i64 that is already scaled.
    #[must_use]
    pub const fn from_scaled_i64(val: i64) -> Self {
        Score(val)
    }

    /// Converts the Score back to a float, removing scaling.
    #[must_use]
    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / SCORE_SCALE
    }

    /// Checked addition.
    #[must_use]
    pub fn checked_add(self, other: Score) -> Option<Score> {
        self.0.checked_add(other.0).map(Score)
    }

    /// Checked subtraction.
    #[must_use]
    pub fn checked_sub(self, other: Score) -> Option<Score> {
        self.0.checked_sub(other.0).map(Score)
    }

    /// Checked multiplication by a scalar.
    #[must_use]
    pub fn checked_mul(self, factor: i64) -> Option<Score> {
        self.0.checked_mul(factor).map(Score)
    }

    /// Saturating addition.
    #[must_use]
    pub fn saturating_add(self, other: Score) -> Score {
        Score::from_scaled_i64(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction.
    #[must_use]
    pub fn saturating_sub(self, other: Score) -> Score {
        Score::from_scaled_i64(self.0.saturating_sub(other.0))
    }

    /// Saturating multiplication.
    #[must_use]
    pub fn saturating_mul(self, factor: i64) -> Score {
        Score::from_scaled_i64(self.0.saturating_mul(factor))
    }
}

impl Add for Score {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl Sub for Score {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

impl Mul<i64> for Score {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        self.saturating_mul(rhs)
    }
}

impl Neg for Score {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Score::from_scaled_i64(self.0.saturating_neg())
    }
}

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.4}", self.to_f32())
    }
}

/// Represents a relative weight for a scoring metric.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Weight(pub f32);

impl Weight {
    /// Zero weight.
    pub const ZERO: Weight = Weight(0.0);

    /// Converts the weight to a float.
    #[must_use]
    pub fn to_f32(self) -> f32 {
        self.0
    }
}

impl From<f32> for Weight {
    fn from(val: f32) -> Self {
        Self(val)
    }
}