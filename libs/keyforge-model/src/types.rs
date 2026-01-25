// libs/keyforge-model/src/types.rs

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

use crate::constants::SCORE_SCALE;
use crate::layout::Layout;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// Unique identifier for a physical key position on the keyboard.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyIndex(pub u16);

impl KeyIndex {
    /// Creates a new `KeyIndex`.
    #[must_use]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    /// Returns the raw `u16` value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Returns the value as `usize`.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for KeyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<usize> for KeyIndex {
    #[allow(clippy::cast_possible_truncation)]
    fn from(idx: usize) -> Self {
        KeyIndex(idx as u16)
    }
}
impl From<KeyIndex> for usize {
    fn from(idx: KeyIndex) -> Self {
        idx.0 as usize
    }
}

/// Logical identifier for a character or action (e.g., 'A', 'Shift').
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyCode(pub u16);

impl KeyCode {
    /// Creates a new `KeyCode`.
    #[must_use]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    /// Returns the raw `u16` value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// The canonical "Empty" or "No-Op" keycode (0).
    pub const EMPTY: KeyCode = KeyCode(0);
    /// The canonical "Transparent" keycode (1).
    /// Used in multi-layer layouts to fall through to the layer below.
    pub const TRANSPARENT: KeyCode = KeyCode(1);
}

impl From<u16> for KeyCode {
    fn from(val: u16) -> Self {
        Self(val)
    }
}
impl From<KeyCode> for u16 {
    fn from(val: KeyCode) -> u16 {
        val.0
    }
}
impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a hand (Left=0, Right=1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct HandIndex(pub u8);

impl HandIndex {
    /// Left hand index (0).
    pub const LEFT: Self = Self(0);
    /// Right hand index (1).
    pub const RIGHT: Self = Self(1);

    /// Creates a new `HandIndex`.
    #[must_use]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    /// Returns the raw `u8` value.
    #[must_use]
    pub fn raw(&self) -> u8 {
        self.0
    }
    /// Returns the raw `u8` value.
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    /// Returns the value as `usize`.
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
    /// Returns true if this is the left hand.
    #[must_use]
    pub fn is_left(&self) -> bool {
        self.0 == 0
    }
    /// Returns true if this is the right hand.
    #[must_use]
    pub fn is_right(&self) -> bool {
        self.0 == 1
    }
}
impl Default for HandIndex {
    fn default() -> Self {
        Self::LEFT
    }
}
impl TryFrom<u8> for HandIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 {
            Err(format!("Invalid HandIndex: {value}"))
        } else {
            Ok(Self(value))
        }
    }
}

/// Identifier for a finger (Thumb=0 to Pinky=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct FingerIndex(pub u8);

impl FingerIndex {
    /// Thumb index (0).
    pub const THUMB: Self = Self(0);
    /// Index finger index (1).
    pub const INDEX: Self = Self(1);
    /// Middle finger index (2).
    pub const MIDDLE: Self = Self(2);
    /// Ring finger index (3).
    pub const RING: Self = Self(3);
    /// Pinky finger index (4).
    pub const PINKY: Self = Self(4);

    /// Creates a new `FingerIndex`.
    #[must_use]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    /// Creates a new `FingerIndex`.
    ///
    /// # Safety
    /// Calling this with a value > 4 violates domain invariants.
    #[must_use]
    pub const fn new_unchecked(val: u8) -> Self {
        Self(val)
    }

    /// Returns the raw `u8` value.
    #[must_use]
    pub fn raw(&self) -> u8 {
        self.0
    }
    /// Returns the raw `u8` value.
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    /// Returns the value as `usize`.
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Calculates the absolute distance between two fingers.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn distance(&self, other: Self) -> u8 {
        (self.0 as i8 - other.0 as i8).unsigned_abs()
    }

    /// Calculates the signed difference between two fingers.
    #[must_use]
    pub fn diff(&self, other: Self) -> i16 {
        i16::from(self.0) - i16::from(other.0)
    }

    /// Returns true if this is considered a "weak" finger (Ring or Pinky).
    #[must_use]
    pub fn is_weak(&self) -> bool {
        self.0 == 3 || self.0 == 4
    }
}
impl Default for FingerIndex {
    fn default() -> Self {
        Self::INDEX
    }
}
impl TryFrom<u8> for FingerIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 {
            Err(format!("Invalid FingerIndex: {value}"))
        } else {
            Ok(Self(value))
        }
    }
}

use serde::de::{self, Visitor};

/// Row index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct RowIndex(pub i8);

impl RowIndex {
    /// Creates a new `RowIndex`.
    #[must_use]
    pub const fn new(val: i8) -> Self {
        Self(val)
    }

    /// Returns the raw `i8` value.
    #[must_use]
    pub const fn raw(self) -> i8 {
        self.0
    }

    /// Returns the value as `usize`.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl<'de> Deserialize<'de> for RowIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RowIndexVisitor;

        impl Visitor<'_> for RowIndexVisitor {
            type Value = RowIndex;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("integer or string representing row index")
            }

            fn visit_i64<E>(self, value: i64) -> Result<RowIndex, E>
            where
                E: de::Error,
            {
                if let Ok(val) = i8::try_from(value) {
                    Ok(RowIndex(val))
                } else {
                    Err(E::custom(format!("RowIndex out of bounds: {value}")))
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<RowIndex, E>
            where
                E: de::Error,
            {
                if let Ok(val) = i8::try_from(value) {
                    Ok(RowIndex(val))
                } else {
                    Err(E::custom(format!("RowIndex out of bounds: {value}")))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<RowIndex, E>
            where
                E: de::Error,
            {
                use std::str::FromStr;
                RowIndex::from_str(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(RowIndexVisitor)
    }
}

impl From<RowIndex> for i8 {
    fn from(idx: RowIndex) -> i8 {
        idx.0
    }
}

impl std::str::FromStr for RowIndex {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.strip_prefix('r').unwrap_or(s);
        clean
            .parse::<i8>()
            .map(RowIndex)
            .map_err(|e| format!("Invalid RowIndex '{s}': {e}"))
    }
}

impl std::ops::Sub for RowIndex {
    type Output = i32;
    fn sub(self, rhs: Self) -> Self::Output {
        i32::from(self.0) - i32::from(rhs.0)
    }
}

/// Column index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct ColIndex(pub i8);

impl From<ColIndex> for i8 {
    fn from(idx: ColIndex) -> i8 {
        idx.0
    }
}

impl std::str::FromStr for ColIndex {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.strip_prefix('c').unwrap_or(s);
        clean
            .parse::<i8>()
            .map(ColIndex)
            .map_err(|e| format!("Invalid ColIndex '{s}': {e}"))
    }
}

impl std::ops::Sub for ColIndex {
    type Output = i32;
    fn sub(self, rhs: Self) -> Self::Output {
        i32::from(self.0) - i32::from(rhs.0)
    }
}

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
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "bigint"))]
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
    pub fn from_f32(val: f32) -> Result<Self, String> {
        if val.is_nan() {
            return Err("Cannot create Score from NaN".to_string());
        }
        let scaled = f64::from(val) * f64::from(crate::constants::SCORE_SCALE);
        #[allow(clippy::cast_precision_loss)]
        if scaled > i64::MAX as f64 || scaled < i64::MIN as f64 {
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
        self.0 as f32 / SCORE_SCALE
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
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
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

use crate::metrics::{MetricId, MetricSet};

/// Preference for which hand should handle Space keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum SpaceHandPreference {
    /// Only use left hand for space.
    Left,
    /// Only use right hand for space.
    Right,
    /// Use both hands (load balanced).
    #[default]
    Bilateral,
}

/// Represents a specific N-gram that violates a metric threshold.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct MetricViolation {
    /// The keys involved (e.g., "TH").
    pub keys: String,
    /// The cost contribution.
    pub score: f32,
    /// The frequency.
    pub freq: f32,
}

/// Detailed breakdown of a layout's performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct AnalysisReport {
    /// Total weighted score.
    pub score: f32,
    /// Standard metric values.
    #[serde(default)]
    pub metrics: MetricSet,
    /// Top offenders grouped by metric.
    #[serde(default)]
    pub violations: HashMap<MetricId, Vec<MetricViolation>>,

    /// Total finger travel distance.
    pub distance: f32,
    /// Average travel distance per keypress.
    pub travel_per_key: f32,
    /// Total Same Finger Bigram cost.
    pub sfb_total: f32,
    /// Ratio of SFBs to total bigrams.
    pub sfb_ratio: f32,
    /// Hand balance (-1.0 Left, +1.0 Right, 0.0 Balanced).
    pub hand_balance: f32,
    /// Scissor score.
    pub scissors: f32,
    /// Redirect score.
    pub redirects: f32,
    /// Inward roll score.
    pub rolls: f32,
    /// Total SFB penalty.
    #[serde(default)]
    pub sfb_penalty: f32,
    /// Total scissor penalty.
    #[serde(default)]
    pub scissor_penalty: f32,
    /// Total redirect penalty.
    #[serde(default)]
    pub redir_penalty: f32,
    /// Total roll penalty.
    #[serde(default)]
    pub roll_penalty: f32,
    /// Usage heatmap.
    #[serde(default)]
    pub heatmap: Vec<f32>,
    /// Effort heatmap.
    #[serde(default)]
    pub penalty_map: Vec<f32>,
    /// Top SFB offenders.
    #[serde(default)]
    pub top_sfbs: Vec<MetricViolation>,
    /// Top Scissor offenders.
    #[serde(default)]
    pub top_scissors: Vec<MetricViolation>,
    /// Top Redirect offenders.
    #[serde(default)]
    pub top_redirs: Vec<MetricViolation>,
}

/// The final output of an optimization run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct OptimizationResult {
    /// The final score achieved (normalized f32).
    pub score: f32,
    /// The raw scaled score (fixed-point i64).
    #[serde(default)]
    pub raw_score: i64,
    /// The optimized layout.
    pub layout: Layout,
}

/// Result of a static scoring operation.
pub type ScoringResult = OptimizationResult;

/// A proposed change to the layout during optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct SwapSuggestion {
    /// Index of the first key.
    pub index_a: usize,
    /// Index of the second key.
    pub index_b: usize,
    /// Label of the first key.
    pub key_a: String,
    /// Label of the second key.
    pub key_b: String,
    /// Change in score (negative is improvement).
    pub score_delta: f32,
    /// Percentage improvement.
    pub improvement_pct: f32,
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_score_overflow_saturation() {
        let max = Score::MAX;
        // Saturating add
        assert_eq!(max + Score(1), Score::MAX);
        // Saturating sub
        assert_eq!(Score::MIN - Score(1), Score::MIN);
        // Saturating mul
        assert_eq!(max * 2, Score::MAX);
    }

    #[test]
    fn test_score_checked_ops() {
        let max = Score::MAX;
        assert!(max.checked_add(Score(1)).is_none());
        assert!(Score::MIN.checked_sub(Score(1)).is_none());
    }

    #[test]
    fn test_score_scaling() {
        let s = Score::from_f32(1.0).unwrap();
        assert_eq!(s.to_f32(), 1.0);
        assert_eq!(s.0, SCORE_SCALE as i64);
    }

    #[test]
    fn test_hand_index_try_from() {
        assert!(HandIndex::try_from(0).is_ok());
        assert!(HandIndex::try_from(1).is_ok());
        assert!(HandIndex::try_from(2).is_err());
    }

    #[test]
    fn test_basic_types_coverage() {
        // KeyIndex
        let ki = KeyIndex(10);
        assert_eq!(format!("{ki}"), "10");
        assert_eq!(usize::from(ki), 10);
        assert_eq!(KeyIndex::from(10usize), ki);

        // KeyCode
        let kc = KeyCode(97);
        assert_eq!(format!("{kc}"), "97");
        assert_eq!(u16::from(kc), 97);
        assert_eq!(KeyCode::from(97u16), kc);

        // HandIndex
        let hi = HandIndex::LEFT;
        assert_eq!(hi.as_u8(), 0);
        assert_eq!(hi.as_usize(), 0);
        assert!(hi.is_left());
        assert!(!hi.is_right());
        assert!(HandIndex::RIGHT.is_right());
        assert_eq!(HandIndex::default(), HandIndex::LEFT);

        // FingerIndex
        let fi = FingerIndex::INDEX;
        assert_eq!(fi.as_u8(), 1);
        assert_eq!(fi.as_usize(), 1);
        assert_eq!(fi.distance(FingerIndex::PINKY), 3);
        assert_eq!(fi.diff(FingerIndex::PINKY), -3);
        assert!(!fi.is_weak());
        assert!(FingerIndex::RING.is_weak());
        assert_eq!(FingerIndex::default(), FingerIndex::INDEX);
        assert!(FingerIndex::try_from(1).is_ok());
        assert!(FingerIndex::try_from(5).is_err());

        // Row/Col Index
        assert_eq!(RowIndex(5) - RowIndex(2), 3);
        assert_eq!(ColIndex(5) - ColIndex(2), 3);
        assert_eq!(RowIndex::default().0, 0);
        assert_eq!(ColIndex::default().0, 0);

        // SpaceHandPreference
        assert_eq!(
            SpaceHandPreference::default(),
            SpaceHandPreference::Bilateral
        );
    }

    #[test]
    fn test_score_extended() {
        assert_eq!(Score::ZERO.0, 0);
        assert_eq!(Score::MAX.0, i64::MAX);
        assert_eq!(Score::MIN.0, i64::MIN);

        let s = Score::from_scaled_i64(100);
        assert_eq!(s.0, 100);

        // Score::from_f32 errors
        assert!(Score::from_f32(f32::NAN).is_err());
        assert!(Score::from_f32(1e20).is_err()); // Overflow

        // Score Ops
        let s1 = Score(100);
        let s2 = Score(50);
        assert_eq!((s1 + s2).0, 150);
        assert_eq!((s1 - s2).0, 50);
        assert_eq!((-s1).0, -100);
        assert_eq!((s1 * 2).0, 200);
    }
}
