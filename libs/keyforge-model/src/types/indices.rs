// libs/keyforge-model/src/types/indices.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// Unique identifier for a physical key position on the keyboard.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyIndex(pub u16);

impl KeyIndex {
    /// Sentinel value for "Not Found" or "Unitialized".
    pub const SENTINEL: KeyIndex = KeyIndex(65535);

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
    fn from(idx: usize) -> Self {
        KeyIndex(idx.try_into().unwrap_or_default())
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
