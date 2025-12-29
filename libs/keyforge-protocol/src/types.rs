use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// Represents a unique index of a key in the geometry vector.
/// Uses u16 to ensure cross-platform determinism (unlike usize).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyIndex(pub u16);

impl fmt::Display for KeyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for KeyIndex {
    fn from(idx: usize) -> Self {
        KeyIndex(idx as u16)
    }
}

impl From<KeyIndex> for usize {
    fn from(idx: KeyIndex) -> Self {
        idx.0 as usize
    }
}

/// Represents a hand (0 = Left, 1 = Right).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[repr(transparent)]
pub struct HandIndex(pub u8);

impl HandIndex {
    pub const LEFT: Self = Self(0);
    pub const RIGHT: Self = Self(1);
}

impl Default for HandIndex {
    fn default() -> Self {
        Self::LEFT
    }
}

/// Represents a finger (0=Thumb, 1=Index, 2=Middle, 3=Ring, 4=Pinky).
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
}

impl Default for FingerIndex {
    fn default() -> Self {
        Self::INDEX
    }
}

/// Physical row index (usually -1 to 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct RowIndex(pub i8);

/// Physical column index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ColIndex(pub i8);
