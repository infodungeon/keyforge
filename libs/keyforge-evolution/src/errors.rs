use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvolutionError {
    #[error("Physics Violation: {0}")]
    Physics(#[from] keyforge_physics::PhysicsError),

    #[error("Configuration Error: {0}")]
    Config(String),
}
