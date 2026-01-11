// libs/keyforge-evolution/src/errors.rs

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

//! Error types for the evolution engine.

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

    /// Optimization was aborted by the user or system via a callback.
    #[error("Optimization Aborted")]
    Aborted,
}
