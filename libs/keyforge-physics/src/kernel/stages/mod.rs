pub(crate) mod corpus;
pub(crate) mod costs;
pub(crate) mod geometry;

use crate::error::PhysicsError;

/// Trait for a discrete stage in the engine compilation pipeline.
pub(crate) trait CompilationStage {
    type Input;
    type Output;
    /// Executes the stage.
    ///
    /// # Errors
    /// Returns `PhysicsError` if the stage execution fails.
    fn execute(&self, input: Self::Input) -> Result<Self::Output, PhysicsError>;
}
