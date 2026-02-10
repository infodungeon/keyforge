use thiserror::Error;

/// Errors related to boundary logic and cross-cutting invariants.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum BoundaryError {
    /// An invariant of the boundary type was violated.
    #[error("Boundary Invariant Violation: {0}")]
    Invariant(String),
}

/// A result type for boundary operations.
pub type BoundaryResult<T> = Result<T, BoundaryError>;
