// libs/keyforge-physics/src/kernel/types.rs

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

use crate::error::PhysicsError;
pub use keyforge_model::types::{
    ColIndex, FingerIndex, HandIndex, KeyCode, KeyIndex, RowIndex, Score,
};

#[derive(Debug, Clone, Copy)]
pub struct ValidatedLayout<'a> {
    slice: &'a [KeyCode],
}

impl<'a> ValidatedLayout<'a> {
    pub fn new(slice: &'a [KeyCode], required_count: usize) -> Result<Self, PhysicsError> {
        if slice.len() < required_count {
            Err(PhysicsError::LayoutUnderflow(slice.len(), required_count))
        } else {
            Ok(Self { slice })
        }
    }
    pub fn as_slice(&self) -> &'a [KeyCode] {
        self.slice
    }
}
