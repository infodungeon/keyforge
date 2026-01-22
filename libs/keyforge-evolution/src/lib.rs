// libs/keyforge-evolution/src/lib.rs

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

//! # `KeyForge` Evolution
//!
//! The optimization engine for `KeyForge`. This crate implements meta-heuristic
//! search algorithms (like Simulated Annealing) to evolve keyboard layouts
//! toward a minimum score.

pub use errors::EvolutionError;
pub mod errors;
pub mod supervisor;

use keyforge_model::KeyCode;

pub use supervisor::{evolve, optimize, optimize_with_callback};

/// Controls the execution of the optimization loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationControl {
    /// Continue with the next step.
    Continue,
    /// Stop optimization gracefully and return the current best result.
    Stop,
    /// Abort immediately without returning a result.
    Abort,
}

/// Trait for receiving progress updates during optimization.
pub trait ProgressCallback: Send + Sync {
    /// Called periodically with the current optimization state.
    fn on_progress(
        &self,
        epoch: usize,
        score: f32,
        layout: &[KeyCode],
        ips: f32,
    ) -> OptimizationControl;
}

/// A progress callback that does nothing.
#[derive(Debug)]
pub struct NoOpCallback;
impl ProgressCallback for NoOpCallback {
    fn on_progress(
        &self,
        _epoch: usize,
        _score: f32,
        _layout: &[KeyCode],
        _ips: f32,
    ) -> OptimizationControl {
        OptimizationControl::Continue
    }
}
