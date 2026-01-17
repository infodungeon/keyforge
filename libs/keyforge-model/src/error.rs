// libs/keyforge-model/src/error.rs

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


//! Centralized error types for the domain.

use thiserror::Error;
use serde::{Serialize, Deserialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// The central error type for the KeyForge domain.
#[derive(Error, Debug)]
pub enum ForgeError {
    /// Input/Output error.
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/Deserialization error.
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Error originating from the Physics engine.
    #[error("Physics Violation: {0}")]
    Physics(#[from] PhysicsError),

    /// Error originating from the Evolution engine.
    #[error("Evolution Error: {0}")]
    Evolution(String),

    /// Error originating from the Persistence layer.
    #[error("Persistence Error: {0}")]
    Persistence(String),

    /// Data validation error.
    #[error("Validation Error: {0}")]
    Validation(String),

    /// Resource not found.
    #[error("Asset Not Found: {0}")]
    NotFound(String),

    /// Internal system error.
    #[error("Internal Error: {0}")]
    Internal(String),

    /// Invalid data format or content.
    #[error("Invalid Data: {0}")]
    InvalidData(String),

    /// Configuration error.
    #[error("Configuration Error: {0}")]
    Config(String),

    /// Error originating from the Model logic itself.
    #[error("Model Error: {0}")]
    Model(#[from] ModelError),
}

/// Errors related to core model logic and integrity.
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum ModelError {
    /// Failed to serialize a component.
    #[error("Serialization Failed: {0}")]
    Serialization(String),
}

/// Specific errors related to physical constraints and scoring.
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum PhysicsError {
    /// Hand index out of bounds (must be 0 or 1).
    #[error("Hand index {0} is invalid (must be 0 or 1)")]
    InvalidHandIndex(u8),

    /// Finger index out of bounds (must be 0-4).
    #[error("Finger index {0} is invalid (must be 0-4)")]
    InvalidFingerIndex(u8),

    /// Matrix dimensions do not match expected values.
    #[error("Matrix dimension mismatch: expected {expected}, found {found}")]
    DimensionMismatch { 
        /// Expected dimension size.
        expected: usize, 
        /// Actual dimension size found.
        found: usize 
    },
    
    /// Layout has more keys than the keyboard.
    #[error("Layout size {0} exceeds physical key count {1}")]
    LayoutOverflow(usize, usize),

    /// Layout has fewer keys than the keyboard.
    #[error("Layout size {0} is insufficient for physical key count {1}")]
    LayoutUnderflow(usize, usize),

    /// Configuration error in the physics engine.
    #[error("Physics Config Error: {0}")]
    Config(String),
}
