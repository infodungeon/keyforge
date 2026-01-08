// libs/keyforge-adapter/src/lib.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # KeyForge Adapter
//!
//! Translation layer between different domain models and external systems. 
//! This crate handles conversions between protocol DTOs and internal domain entities.

/// Conversion logic between protocol/UI types and domain models.
pub mod conversion;
/// Crate-specific error and result types.
pub mod error;
pub mod parsing;

pub use error::{AdapterError, AdapterResult};
