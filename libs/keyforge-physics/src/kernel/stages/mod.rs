pub mod corpus;
pub mod costs;
pub mod geometry;

use crate::errors::PhysicsError;

/// Trait for a discrete stage in the engine compilation pipeline.
pub trait CompilationStage {
    type Input;
    type Output;
    fn execute(&self, input: Self::Input) -> Result<Self::Output, PhysicsError>;
}
