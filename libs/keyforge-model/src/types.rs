use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use crate::constants::SCORE_SCALE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyIndex(pub u16);

impl fmt::Display for KeyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl From<usize> for KeyIndex { fn from(idx: usize) -> Self { KeyIndex(idx as u16) } }
impl From<KeyIndex> for usize { fn from(idx: KeyIndex) -> Self { idx.0 as usize } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyCode(pub u16);

impl From<u16> for KeyCode { fn from(val: u16) -> Self { Self(val) } }
impl From<KeyCode> for u16 { fn from(val: KeyCode) -> u16 { val.0 } }
impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct HandIndex(pub u8);

impl HandIndex {
    pub const LEFT: Self = Self(0);
    pub const RIGHT: Self = Self(1);
    pub fn as_u8(&self) -> u8 { self.0 }
    pub fn as_usize(&self) -> usize { self.0 as usize }
}
impl Default for HandIndex { fn default() -> Self { Self::LEFT } }
impl TryFrom<u8> for HandIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 { Err(format!("Invalid HandIndex: {}", value)) } else { Ok(Self(value)) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct FingerIndex(pub u8);

impl FingerIndex {
    pub const THUMB: Self = Self(0);
    pub const INDEX: Self = Self(1);
    pub const MIDDLE: Self = Self(2);
    pub const RING: Self = Self(3);
    pub const PINKY: Self = Self(4);
    pub fn as_u8(&self) -> u8 { self.0 }
    pub fn as_usize(&self) -> usize { self.0 as usize }
}
impl Default for FingerIndex { fn default() -> Self { Self::INDEX } }
impl TryFrom<u8> for FingerIndex {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 { Err(format!("Invalid FingerIndex: {}", value)) } else { Ok(Self(value)) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct RowIndex(pub i8);

impl std::ops::Sub for RowIndex {
    type Output = i8;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ColIndex(pub i8);

impl std::ops::Sub for ColIndex {
    type Output = i8;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Score(pub i64);

impl Score {
    pub const MAX: Score = Score(i64::MAX);
    pub const MIN: Score = Score(i64::MIN);
    pub const ZERO: Score = Score(0);
    pub fn from_f32(val: f32) -> Self {
        if val.is_nan() { return Self::ZERO; }
        if val.is_infinite() { return if val.is_sign_positive() { Self::MAX } else { Self::MIN }; }
        let scaled = val * SCORE_SCALE;
        if scaled >= i64::MAX as f32 { return Self::MAX; }
        if scaled <= i64::MIN as f32 { return Self::MIN; }
        Self(scaled as i64)
    }
    pub fn to_f32(self) -> f32 { self.0 as f32 / SCORE_SCALE }
    pub fn saturating_add(self, other: Score) -> Score { Score(self.0.saturating_add(other.0)) }
    pub fn saturating_sub(self, other: Score) -> Score { Score(self.0.saturating_sub(other.0)) }
    pub fn saturating_mul(self, factor: i64) -> Score { Score(self.0.saturating_mul(factor)) }
}
impl std::ops::Add for Score { type Output = Self; fn add(self, rhs: Self) -> Self::Output { self.saturating_add(rhs) } }
impl std::ops::Sub for Score { type Output = Self; fn sub(self, rhs: Self) -> Self::Output { self.saturating_sub(rhs) } }
