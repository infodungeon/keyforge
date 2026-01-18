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


use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use crate::constants::SCORE_SCALE;

/// Unique identifier for a physical key position on the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyIndex(pub u16);

impl fmt::Display for KeyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl From<usize> for KeyIndex { 
    #[allow(clippy::cast_possible_truncation)]
    fn from(idx: usize) -> Self { KeyIndex(idx as u16) } 
}
impl From<KeyIndex> for usize { fn from(idx: KeyIndex) -> Self { idx.0 as usize } }

/// Logical identifier for a character or action (e.g., 'A', 'Shift').
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyCode(pub u16);

impl From<u16> for KeyCode { fn from(val: u16) -> Self { Self(val) } }
impl From<KeyCode> for u16 { fn from(val: KeyCode) -> u16 { val.0 } }
impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
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
    
    /// Returns the raw `u8` value.
    #[must_use] 
    pub fn as_u8(&self) -> u8 { self.0 }
    /// Returns the value as `usize`.
    #[must_use] 
    pub fn as_usize(&self) -> usize { self.0 as usize }
    /// Returns true if this is the left hand.
    #[must_use] 
    pub fn is_left(&self) -> bool { self.0 == 0 }
    /// Returns true if this is the right hand.
    #[must_use] 
    pub fn is_right(&self) -> bool { self.0 == 1 }
}
impl Default for HandIndex { fn default() -> Self { Self::LEFT } }
impl TryFrom<u8> for HandIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 { Err(format!("Invalid HandIndex: {value}")) } else { Ok(Self(value)) }
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
    
    /// Returns the raw `u8` value.
    #[must_use] 
    pub fn as_u8(&self) -> u8 { self.0 }
    /// Returns the value as `usize`.
    #[must_use] 
    pub fn as_usize(&self) -> usize { self.0 as usize }
    
    /// Calculates the absolute distance between two fingers.
    #[must_use] 
    #[allow(clippy::cast_possible_wrap)]
    pub fn distance(&self, other: Self) -> u8 {
        (self.0 as i8 - other.0 as i8).unsigned_abs()
    }
    
    /// Calculates the signed difference between two fingers.
    #[must_use] 
    #[allow(clippy::cast_possible_wrap)]
    pub fn diff(&self, other: Self) -> i8 {
        self.0 as i8 - other.0 as i8
    }
    
    /// Returns true if this is considered a "weak" finger (Ring or Pinky).
    #[must_use] 
    pub fn is_weak(&self) -> bool {
        self.0 == 3 || self.0 == 4
    }
}
impl Default for FingerIndex { fn default() -> Self { Self::INDEX } }
impl TryFrom<u8> for FingerIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 { Err(format!("Invalid FingerIndex: {value}")) } else { Ok(Self(value)) }
    }
}

/// Row index.
///
/// Typically: Home=0, Top<0, Bottom>0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct RowIndex(pub i8);

impl std::ops::Sub for RowIndex {
    type Output = i8;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}

/// Column index.
///
/// Positive values are typically to the right, negative to the left, 
/// with 0 being a reference column (e.g., center or pinky column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct ColIndex(pub i8);

impl std::ops::Sub for ColIndex {
    type Output = i8;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}

/// Fixed-point score value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
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

    /// Creates a Score from a float value, applying scaling.
    #[must_use] 
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_f32(val: f32) -> Self {
        Score((val * SCORE_SCALE) as i64)
    }

    /// Creates a Score from a raw i64 that is already scaled.
    /// Use this when deserializing or loading pre-calculated values.
    #[must_use] 
    pub fn from_scaled_i64(val: i64) -> Self {
        Score(val)
    }

    /// Converts the Score back to a float, removing scaling.
    #[must_use] 
    #[allow(clippy::cast_precision_loss)]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / SCORE_SCALE
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
    
    /// Saturating multiplication by an integer factor.
    #[must_use] 
    pub fn saturating_mul(self, factor: i64) -> Score {
        Score(self.0.saturating_mul(factor))
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