// libs/keyforge-model/src/mapping.rs

use crate::error::ForgeError;

/// A trait for projecting domain models from external representations (SQLX, JSON, etc).
///
/// This facilitates the "Correct-by-Construction" pattern by centralizing
/// mapping logic and ensuring validation during conversion.
pub trait Projection<Source>: Sized {
    /// Projects a source into the target model.
    ///
    /// # Errors
    /// Returns `ForgeError::Projection` if mapping or validation fails.
    fn project(source: Source) -> Result<Self, ForgeError>;
}

/// Helper for bulk projections.
pub trait BulkProjection<Source>: Sized {
    /// Projects a collection of sources.
    fn project_all(sources: Vec<Source>) -> Result<Vec<Self>, ForgeError>;
}

impl<T, S> BulkProjection<S> for T
where
    T: Projection<S>,
{
    fn project_all(sources: Vec<S>) -> Result<Vec<Self>, ForgeError> {
        sources.into_iter().map(T::project).collect()
    }
}
