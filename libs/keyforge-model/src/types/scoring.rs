// libs/keyforge-model/src/types/scoring.rs

use crate::constants::{SCORE_SCALE, WEIGHT_SCALE};
use crate::types::FixedPointMath;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// Represents a biomechanical effort score in deterministic fixed-point units.
///
/// Scaling: 1,000,000 units = 1.0 Effort Point.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, Default,
)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
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
    #[allow(clippy::cast_precision_loss)]
    fn scale() -> f64 {
        // SAFETY: TYPE-001 Exception: Precision-aware scaling factor.
        f64::from(1_000_000i32)
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
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn from_f32(val: f32) -> Result<Self, String> {
        if val.is_nan() {
            return Err("Cannot create Score from NaN".to_string());
        }
        let score_scale_f64 = f64::from(1_000_000i32);
        let scaled = f64::from(val) * score_scale_f64;
        if scaled > (i64::MAX as f64) || scaled < (i64::MIN as f64) {
            // sg-ignore
            return Err(format!(
                "Score overflow: {val} * {SCORE_SCALE} exceeds i64 range"
            ));
        }
        // SAFETY: TYPE-001 Exception: Physics-aware conversion to scaled fixed-point.
        Ok(Score::from_scaled_i64(scaled as i64)) // sg-ignore
    }

    /// Creates a Score from a raw i64 that is already scaled.
    #[must_use]
    pub const fn from_scaled_i64(val: i64) -> Self {
        Score(val)
    }

    /// Creates a Score from a `FixedWeight`.
    #[must_use]
    pub fn from_weight(weight: FixedWeight) -> Self {
        // WEIGHT_SCALE is 1,000, SCORE_SCALE is 1,000,000.
        // Ratio is 1,000.
        let ratio = SCORE_SCALE / i64::from(WEIGHT_SCALE);
        Self::from_scaled_i64(i64::from(weight.raw()) * ratio)
    }

    /// Converts the Score back to a float, removing scaling.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn to_f32(self) -> f32 {
        // SAFETY: TYPE-001 Exception: Physics-aware conversion back to float.
        (self.0 as f32) / 1_000_000.0 // sg-ignore
    }

    /// Checked addition.
    #[must_use]
    pub fn checked_add(self, other: Score) -> Option<Score> {
        self.0.checked_add(other.raw()).map(Score)
    }

    /// Checked subtraction.
    #[must_use]
    pub fn checked_sub(self, other: Score) -> Option<Score> {
        self.0.checked_sub(other.raw()).map(Score)
    }

    /// Checked multiplication by a scalar.
    #[must_use]
    pub fn checked_mul(self, factor: i64) -> Option<Score> {
        self.0.checked_mul(factor).map(Score)
    }

    /// Saturating addition.
    #[must_use]
    pub fn saturating_add(self, other: Score) -> Score {
        Score::from_scaled_i64(self.0.saturating_add(other.raw()))
    }

    /// Saturating subtraction.
    #[must_use]
    pub fn saturating_sub(self, other: Score) -> Score {
        Score::from_scaled_i64(self.0.saturating_sub(other.raw()))
    }

    /// Saturating multiplication.
    #[must_use]
    pub fn saturating_mul(self, factor: i64) -> Score {
        Score::from_scaled_i64(self.0.saturating_mul(factor))
    }

    /// Checked multiplication by a `FixedWeight`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn checked_mul_weight(self, weight: FixedWeight) -> Option<Score> {
        let raw_score = i128::from(self.raw());
        let raw_weight = i128::from(weight.raw());
        let scaled = (raw_score.checked_mul(raw_weight)?) / i128::from(WEIGHT_SCALE);
        if scaled > i128::from(i64::MAX) || scaled < i128::from(i64::MIN) {
            None
        } else {
            // SAFETY: TYPE-001 Exception: Physics-aware fixed-point math.
            Some(Score::from_scaled_i64(scaled as i64)) // sg-ignore
        }
    }

    /// Saturating multiplication by a `FixedWeight`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn saturating_mul_weight(self, weight: FixedWeight) -> Score {
        let raw_score = i128::from(self.raw());
        let raw_weight = i128::from(weight.raw());
        let scaled = (raw_score.saturating_mul(raw_weight)) / i128::from(WEIGHT_SCALE);
        // SAFETY: TYPE-001 Exception: Physics-aware fixed-point math.
        Score::from_scaled_i64(scaled.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
        // sg-ignore
    }

    /// Performs deterministic normalization (e.g., Score per 100k keys).
    /// Formula: (Accumulated * Scale + (Divisor / 2)) / Divisor
    ///
    /// # Errors
    /// Returns an error if the divisor is zero.
    #[allow(clippy::cast_possible_truncation)]
    pub fn normalized(self, scale: i64, divisor: u64) -> Result<Self, String> {
        if divisor == 0 {
            return Err("Division by zero in normalization".to_string());
        }
        let accumulated = i128::from(self.raw());
        let scale_128 = i128::from(scale);
        let divisor_128 = i128::from(divisor);

        let product = accumulated * scale_128;
        let rounding = if product >= 0 {
            divisor_128 / 2
        } else {
            -(divisor_128 / 2)
        };

        let result = (product + rounding) / divisor_128;

        if result > i128::from(i64::MAX) || result < i128::from(i64::MIN) {
            return Err("Normalization overflowed i64".to_string());
        }

        // SAFETY: TYPE-001 Exception: Physics-aware fixed-point normalization.
        Ok(Score::from_scaled_i64(result as i64)) // sg-ignore
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

impl Mul<FixedWeight> for Score {
    type Output = Self;
    fn mul(self, rhs: FixedWeight) -> Self::Output {
        self.saturating_mul_weight(rhs)
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

impl std::iter::Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Score::ZERO, |a, b| a + b)
    }
}

impl<'a> std::iter::Sum<&'a Score> for Score {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Score::ZERO, |a, b| a + *b)
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

/// Represents a relative weight for a scoring metric in deterministic fixed-point units.
///
/// Scaling: 1,000 units = 1.0 Weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, utoipa::ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[repr(transparent)]
pub struct FixedWeight(i32);

impl serde::Serialize for FixedWeight {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.to_f32())
    }
}

impl<'de> serde::Deserialize<'de> for FixedWeight {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = f32::deserialize(deserializer)?;
        Self::from_f32(val).map_err(serde::de::Error::custom)
    }
}

impl FixedPointMath for FixedWeight {
    type Raw = i32;
    fn raw(self) -> Self::Raw {
        self.0
    }
    fn from_raw(val: Self::Raw) -> Self {
        Self(val)
    }
    #[allow(clippy::cast_precision_loss)]
    fn scale() -> f64 {
        // SAFETY: TYPE-001 Exception: Precision-aware scaling factor.
        f64::from(1_000i32)
    }
}

impl std::fmt::Display for FixedWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.to_f32())
    }
}

impl FixedWeight {
    /// Zero weight.
    pub const ZERO: FixedWeight = FixedWeight(0);
    /// Unit weight (1.0).
    pub const UNIT: FixedWeight = FixedWeight(WEIGHT_SCALE);

    /// Creates a `FixedWeight` from a float value, applying scaling.
    ///
    /// # Errors
    /// Returns an error if the resulting value overflows `i32` or is NaN.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_lossless
    )]
    pub fn from_f32(val: f32) -> Result<Self, String> {
        if val.is_nan() {
            return Err("Cannot create FixedWeight from NaN".to_string());
        }
        // SAFETY: TYPE-001 Exception: Physics-aware conversion to scaled fixed-point.
        let scaled = f64::from(val) * f64::from(1_000i32);
        if scaled > f64::from(i32::MAX) || scaled < f64::from(i32::MIN) {
            return Err(format!(
                "Weight overflow: {val} * {WEIGHT_SCALE} exceeds i32 range"
            ));
        }
        // SAFETY: TYPE-001 Exception: Physics-aware conversion to scaled fixed-point.
        Ok(FixedWeight(scaled.round() as i32)) // sg-ignore
    }

    /// Converts the `FixedWeight` back to a float, removing scaling.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn to_f32(self) -> f32 {
        // SAFETY: TYPE-001 Exception: Physics-aware conversion back to float.
        (self.0 as f32) / 1_000.0 // sg-ignore
    }

    /// Returns the raw `i32` value.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }
}
