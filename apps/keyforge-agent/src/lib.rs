//! # KeyForge Agent
//!
//! Distributed compute worker for KeyForge. This crate implements the 
//! logic for remote job processing, hardware capability detection, 
//! and secure communication with the Hive.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


/// Core agent logic including the worker loop and job execution.
pub mod agent; // The Core Agent Logic
/// Hardware capability detection and benchmarking.
pub mod hw_detect;
/// Structured logging and telemetry configuration.
pub mod logging;
/// Data models for agent-hive communication.
pub mod models;


// Re-export the main runner
pub use agent::run_worker;
