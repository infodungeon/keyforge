// libs/keyforge-model/src/types/geometry.rs

use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// Row index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema, Default)]
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
    pub fn as_usize(self) -> usize {
        self.0.try_into().unwrap_or_default()
    }

    /// Calculates the absolute distance between two rows.
    #[must_use]
    pub fn distance(self, other: Self) -> u8 {
        self.0.abs_diff(other.0)
    }

    /// Calculates the signed difference between two rows.
    #[must_use]
    pub fn abs_diff(self, other: Self) -> i8 {
        (i32::from(self.0) - i32::from(other.0)).abs() as i8
    }
}

impl fmt::Display for RowIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
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
#[serde(transparent)]
#[repr(transparent)]
pub struct ColIndex(pub i8);

impl ColIndex {
    /// Creates a new `ColIndex`.
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
    pub fn as_usize(self) -> usize {
        self.0.try_into().unwrap_or_default()
    }

    /// Calculates the absolute distance between two rows.
    #[must_use]
    pub fn distance(self, other: Self) -> u8 {
        self.0.abs_diff(other.0)
    }

    /// Calculates the signed difference between two rows.
    #[must_use]
    pub fn abs_diff(self, other: Self) -> i8 {
        (i32::from(self.0) - i32::from(other.0)).abs() as i8
    }
}

impl fmt::Display for ColIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c{}", self.0)
    }
}

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
