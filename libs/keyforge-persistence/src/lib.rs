// Copyright (c) 2025 KeyForge Contributors
//
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
//! KeyForge Persistence
//!
//! This crate provides the data structures and services for persisting user projects,
//! snapshots, and compiled runtimes.

pub(crate) mod compiler;
pub(crate) mod error;
pub(crate) mod project;
pub(crate) mod store;

/// Compiles a [Project] into an executable [Runtime].
pub use compiler::Compiler;
/// Error types for persistence operations.
pub use error::{PersistenceError, PersistenceResult};
/// Project definition and metadata.
pub use project::{Project, ProjectMeta};
/// Automated background saving service.
pub use store::autosave::{AutoSaveService, SessionSnapshot};
