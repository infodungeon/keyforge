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

impl From<String> for PhysicsError {
    fn from(s: String) -> Self {
        PhysicsError::CalculationError(s)
    }
}
