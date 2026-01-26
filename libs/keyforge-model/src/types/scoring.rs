// libs/keyforge-model/src/types/scoring.rs

use crate::constants::SCORE_SCALE;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Fixed-point score value.
///
/// Uses saturating arithmetic by default to prevent panics during evaluation.
///
/// # Examples
///
/// ```
/// use keyforge_model::Score;
/// let a = Score::MAX;
/// let b = Score(1);
/// assert_eq!(a + b, Score::MAX);
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Score(pub i64);

impl Score {
    /// Maximum possible score.
    pub const MAX: Score = Score(i64::MAX);
    /// Minimum possible score.
    pub const MIN: Score = Score(i64::MIN);
    /// Zero score.
    pub const ZERO: Score = Score(0);
    /// Sentinel value for unreachable or uninitialized costs.
    pub const INFINITY_SENTINEL: Score = Score(i64::MAX);

    /// Creates a Score from a float value, applying scaling.
    ///
    /// # Errors
    /// Returns an error string if the value overflows or is NaN.
    #[allow(clippy::cast_precision_loss)]
    pub fn from_f32(val: f32) -> Result<Self, String> {
        if val.is_nan() {
            return Err("Cannot create Score from NaN".to_string());
        }
        let scaled = f64::from(val) * f64::from(SCORE_SCALE);
        // SAFETY: Bounds check before casting to i64.
        if scaled > (i64::MAX as f64) || scaled < (i64::MIN as f64) {
            return Err(format!(
                "Score overflow: {val} exceeds i64 bounds when scaled"
            ));
        }
        #[allow(clippy::cast_possible_truncation)]
        Ok(Score(scaled as i64))
    }

    /// Creates a Score from a raw i64 that is already scaled.
    #[must_use]
    pub const fn from_scaled_i64(val: i64) -> Self {
        Score(val)
    }

    /// Converts the Score back to a float, removing scaling.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / SCORE_SCALE
    }

    /// Checked addition.
    pub fn checked_add(self, other: Score) -> Option<Score> {
        self.0.checked_add(other.0).map(Score)
    }

    /// Checked subtraction.
    pub fn checked_sub(self, other: Score) -> Option<Score> {
        self.0.checked_sub(other.0).map(Score)
    }

    /// Checked multiplication.
    pub fn checked_mul(self, factor: i64) -> Option<Score> {
        self.0.checked_mul(factor).map(Score)
    }

    /// Saturating addition.
    #[must_use]
    pub fn saturating_add(self, other: Score) -> Score {
        Score(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction.
    #[must_use]
    pub fn saturating_sub(self, other: Score) -> Score {
        Score(self.0.saturating_sub(other.0))
    }

    /// Saturating multiplication.
    #[must_use]
    pub fn saturating_mul(self, factor: i64) -> Score {
        Score(self.0.saturating_mul(factor))
    }
}

/// A wrapper for ergonomic weights and multipliers.
/// Prevents argument swapping and ensures semantic clarity in scoring logic.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Weight(pub f32);

impl Weight {
    /// Zero weight.
    pub const ZERO: Weight = Weight(0.0);

    /// Converts the weight to a raw f32.
    #[must_use]
    pub const fn to_f32(self) -> f32 {
        self.0
    }
}

impl From<f32> for Weight {
    fn from(val: f32) -> Self {
        Weight(val)
    }
}

impl From<Weight> for f32 {
    fn from(w: Weight) -> Self {
        w.0
    }
}

impl std::fmt::Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl std::ops::Mul<f32> for Weight {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Weight(self.0 * rhs)
    }
}

impl std::ops::Add for Weight {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Weight(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Weight {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Weight(self.0 - rhs.0)
    }
}

impl std::ops::Add for Score {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl std::ops::Sub for Score {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

impl std::ops::Neg for Score {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Score(self.0.saturating_neg())
    }
}

impl std::ops::Mul<i64> for Score {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        self.saturating_mul(rhs)
    }
}
