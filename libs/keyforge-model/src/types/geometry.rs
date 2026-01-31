// libs/keyforge-model/src/types/geometry.rs

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

//! Keyboard geometry types.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};
use utoipa::ToSchema;

/// Represents a logical row index on a keyboard.
/// Positive values are usually bottom-to-top, but logical mapping varies by keyboard.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[schema(as = i8)]
pub struct RowIndex(i8);

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
}

impl std::str::FromStr for RowIndex {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = if s.starts_with('r') {
            s[1..].parse()?
        } else {
            s.parse()?
        };
        Ok(Self(val))
    }
}

impl std::fmt::Display for RowIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

impl Add<i8> for RowIndex {
    type Output = Self;
    fn add(self, rhs: i8) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl Sub<RowIndex> for RowIndex {
    type Output = i8;
    fn sub(self, rhs: RowIndex) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}

/// Represents a logical column index on a keyboard.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[schema(as = i8)]
pub struct ColIndex(i8);

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
}

impl std::str::FromStr for ColIndex {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = if s.starts_with('c') {
            s[1..].parse()?
        } else {
            s.parse()?
        };
        Ok(Self(val))
    }
}

impl std::fmt::Display for ColIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

impl Add<i8> for ColIndex {
    type Output = Self;
    fn add(self, rhs: i8) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl Sub<ColIndex> for ColIndex {
    type Output = i8;
    fn sub(self, rhs: ColIndex) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}