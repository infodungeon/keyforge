// Copyright (c) 2025 KeyForge Contributors
//
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

/// Canonical index of a physical key in the keyboard array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyIndex(pub u16);

impl fmt::Display for KeyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl From<usize> for KeyIndex { fn from(idx: usize) -> Self { KeyIndex(idx as u16) } }
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

/// Identifies the hand (Left=0, Right=1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct HandIndex(pub u8);

impl HandIndex {
    /// Left Hand (0).
    pub const LEFT: Self = Self(0);
    /// Right Hand (1).
    pub const RIGHT: Self = Self(1);
    /// Returns the raw u8 value.
    pub fn as_u8(&self) -> u8 { self.0 }
    /// Returns the value as usize.
    pub fn as_usize(&self) -> usize { self.0 as usize }
    /// Checks if Left.
    pub fn is_left(&self) -> bool { self.0 == Self::LEFT.0 }
    /// Checks if Right.
    pub fn is_right(&self) -> bool { self.0 == Self::RIGHT.0 }
}
impl Default for HandIndex { fn default() -> Self { Self::LEFT } }
impl TryFrom<u8> for HandIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 { Err(format!("Invalid HandIndex: {}", value)) } else { Ok(Self(value)) }
    }
}

/// Identifies the finger (Thumb=0, Index=1, Middle=2, Ring=3, Pinky=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct FingerIndex(pub u8);

impl FingerIndex {
    /// Thumb (0).
    pub const THUMB: Self = Self(0);
    /// Index Finger (1).
    pub const INDEX: Self = Self(1);
    /// Middle Finger (2).
    pub const MIDDLE: Self = Self(2);
    /// Ring Finger (3).
    pub const RING: Self = Self(3);
    /// Pinky Finger (4).
    pub const PINKY: Self = Self(4);
    /// Returns the raw u8 value.
    pub fn as_u8(&self) -> u8 { self.0 }
    /// Returns the value as usize.
    pub fn as_usize(&self) -> usize { self.0 as usize }
    /// Calculates the absolute distance between two fingers.
    pub fn distance(&self, other: Self) -> u8 { (self.0 as i8 - other.0 as i8).unsigned_abs() }
    /// Calculates the signed difference between two fingers.
    pub fn diff(&self, other: Self) -> i8 { self.0 as i8 - other.0 as i8 }
}
impl Default for FingerIndex { fn default() -> Self { Self::INDEX } }
impl TryFrom<u8> for FingerIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 { Err(format!("Invalid FingerIndex: {}", value)) } else { Ok(Self(value)) }
    }
}

/// Logical row index (0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct RowIndex(pub i8);

impl std::ops::Sub for RowIndex {
    type Output = i8;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}

/// Logical column index (0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, type = "number"))]
#[serde(transparent)]
#[repr(transparent)]
pub struct ColIndex(pub i8);

impl std::ops::Sub for ColIndex {
    type Output = i8;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}

/// Fixed-point score representation.
/// Internally stored as `i64` scaled by `SCORE_SCALE` (1,000,000).
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
    
    /// Converts an f32 to Score, handling saturation and NaN.
    pub fn from_f32(val: f32) -> Self {
        if val.is_nan() { return Self::ZERO; }
        if val.is_infinite() { return if val.is_sign_positive() { Self::MAX } else { Self::MIN }; }
        let scaled = val * SCORE_SCALE;
        if scaled >= i64::MAX as f32 { return Self::MAX; }
        if scaled <= i64::MIN as f32 { return Self::MIN; }
        Self(scaled as i64)
    }
    
    /// Converts Score back to f32.
    pub fn to_f32(self) -> f32 { self.0 as f32 / SCORE_SCALE }
    
    /// Saturating addition.
    pub fn saturating_add(self, other: Score) -> Score { Score(self.0.saturating_add(other.0)) }
    /// Saturating subtraction.
    pub fn saturating_sub(self, other: Score) -> Score { Score(self.0.saturating_sub(other.0)) }
    /// Saturating multiplication by an integer factor.
    pub fn saturating_mul(self, factor: i64) -> Score { Score(self.0.saturating_mul(factor)) }
}
impl std::ops::Add for Score { type Output = Self; fn add(self, rhs: Self) -> Self::Output { self.saturating_add(rhs) } }
impl std::ops::Sub for Score { type Output = Self; fn sub(self, rhs: Self) -> Self::Output { self.saturating_sub(rhs) } }
