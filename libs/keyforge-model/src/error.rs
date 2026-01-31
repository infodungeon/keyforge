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

//! Centralized error types for the domain nucleus.
//!
//! Following ARCH-005 and ADR-022, this module is pure and contains zero
//! dependencies on infrastructure-layer error types (std::io, serde_json).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The central error type for the `KeyForge` domain.
#[derive(Error, Debug)]
pub enum ForgeError {
    /// High-level physics calculation error from the compute kernel.
    #[error("Physics Compute Error: {0}")]
    PhysicsCompute(String),

    /// Error originating from the Evolution engine.
    #[error("Evolution Error: {0}")]
    Evolution(String),

    /// Error originating from the Persistence layer.
    #[error("Persistence Error: {0}")]
    Persistence(String),

    /// Data validation error (Business Rule Violation).
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

    /// Error originating from JSON serialization/deserialization.
    /// This is now a pure string to avoid coupling to Serde crates in the nucleus.
    #[error("Serde Error: {0}")]
    Serde(String),

    /// Error originating from component serialization.
    #[error("Serialization Error: {0}")]
    Serialization(String),

    /// Configuration error.
    #[error("Configuration Error: {0}")]
    Config(String),

    /// Input/Output error wrapped as a domain string.
    /// ARCH-005: Decoupled from std::io::Error.
    #[error("IO Error: {0}")]
    Io(String),

    /// Error originating during data projection.
    #[error("Projection Error: {0}")]
    Projection(String),

    /// Error originating from the Physics engine.
    #[error("Physics Violation: {0}")]
    Physics(#[from] PhysicsError),

    /// Error originating from the Model logic itself.
    #[error("Model Error: {0}")]
    Model(#[from] ModelError),

    /// Infrastructure-layer error wrapped for domain propagation.
    /// Used at boundaries to preserve diagnostic context without coupling the nucleus.
    #[error("Infrastructure Failure: {0}")]
    Infrastructure(String),
}

impl From<String> for ForgeError {
    fn from(s: String) -> Self {
        ForgeError::Validation(s)
    }
}

/// Errors related to core model logic and integrity.
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModelError {
    /// Failed to serialize a component.
    #[error("Serialization Failed: {0}")]
    Serialization(String),

    /// Failed to parse a keymap or key action.
    #[error("Parser Error: {0}")]
    Parser(String),

    /// An invariant of the domain model was violated.
    #[error("Invariant Violation: {0}")]
    Invariant(String),
}

/// Specific errors related to physical constraints and scoring.
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
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
        found: usize,
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

    /// Feature not implemented.
    #[error("Not Implemented: {0}")]
    Unimplemented(String),
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert!(format!("{}", ForgeError::Internal("test".into())).contains("Internal Error: test"));
        assert!(format!("{}", ModelError::Serialization("test".into()))
            .contains("Serialization Failed: test"));
        assert!(
            format!("{}", PhysicsError::InvalidHandIndex(5)).contains("Hand index 5 is invalid")
        );
        assert!(format!("{}", PhysicsError::InvalidFingerIndex(10))
            .contains("Finger index 10 is invalid"));
        assert!(format!(
            "{}",
            PhysicsError::DimensionMismatch {
                expected: 10,
                found: 5
            }
        )
        .contains("expected 10, found 5"));
        assert!(format!("{}", PhysicsError::LayoutOverflow(10, 5))
            .contains("exceeds physical key count"));
        assert!(format!("{}", PhysicsError::LayoutUnderflow(2, 5)).contains("is insufficient"));
        assert!(format!("{}", PhysicsError::Config("test".into()))
            .contains("Physics Config Error: test"));
        assert!(format!("{}", PhysicsError::Unimplemented("test".into()))
            .contains("Not Implemented: test"));
    }
}
