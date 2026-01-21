// libs/keyforge-physics/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Specialized errors for the physics and scoring engine.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum PhysicsError {
    /// A mathematical overflow occurred during scoring.
    /// This indicates "Bad Code" or "Impossible Data."
    #[error("Score overflow in context: {context}")]
    ScoreOverflow {
        /// Context describing where the overflow happened (e.g., "Bigram(T, H)").
        context: String 
    },

    /// The input data (Keyboard or Corpus) violated physical constraints.
    #[error("Invalid input data: {message}")]
    InvalidInput {
        /// Human-readable explanation of the constraint violation.
        message: String 
    },

    /// Engine configuration or compilation error.
    #[error("Engine configuration error: {0}")]
    Config(String),

    /// Layout size mismatch.
    #[error("Layout size mismatch: expected {1}, found {0}")]
    LayoutUnderflow(usize, usize),

    /// A general calculation error.
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_error_display() {
        let err = PhysicsError::ScoreOverflow { context: "Test".into() };
        assert_eq!(err.to_string(), "Score overflow in context: Test");

        let err = PhysicsError::InvalidInput { message: "Bad keys".into() };
        assert_eq!(err.to_string(), "Invalid input data: Bad keys");

        let err = PhysicsError::Config("Bad config".into());
        assert_eq!(err.to_string(), "Engine configuration error: Bad config");

        let err = PhysicsError::LayoutUnderflow(10, 20);
        assert_eq!(err.to_string(), "Layout size mismatch: expected 20, found 10");

        let err = PhysicsError::CalculationError("Math failed".into());
        assert_eq!(err.to_string(), "Calculation error: Math failed");
    }

    #[test]
    fn test_from_string() {
        let err: PhysicsError = "Something wrong".to_string().into();
        match err {
            PhysicsError::CalculationError(msg) => assert_eq!(msg, "Something wrong"),
            _ => panic!("Expected CalculationError"),
        }
    }

    #[test]
    fn test_debug_derive() {
        let err = PhysicsError::Config("Debug me".into());
        let dbg = format!("{:?}", err);
        assert!(dbg.contains("Config"));
        assert!(dbg.contains("Debug me"));
    }
}
