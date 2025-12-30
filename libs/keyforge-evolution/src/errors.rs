use thiserror::Error;

/// Errors that can occur during the evolution process.
#[derive(Error, Debug)]
pub enum EvolutionError {
    /// Error propagated from the physics engine.
    #[error("Physics Violation: {0}")]
    Physics(#[from] keyforge_physics::PhysicsError),

    /// Error caused by invalid configuration or input parameters.
    #[error("Configuration Error: {0}")]
    Config(String),
}
