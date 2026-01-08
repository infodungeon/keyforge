// libs/keyforge-persistence/src/lib.rs

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

//! # KeyForge Persistence
//!
//! Abstractions for data storage and project management. This crate handles 
//! saving and loading keyboard definitions, corpora, and optimization results.

pub mod error;
pub mod repo;
pub mod project;
pub mod compiler;

// Re-exports for UI/CLI
pub use project::{Project, ProjectMeta};
pub use compiler::Compiler;
pub use error::{PersistenceError, PersistenceResult};

// Re-exports for Hive/Agent
pub use repo::user_repo::UserRepo;
pub mod store;
pub use store::autosave::{AutoSaveService, SessionSnapshot};
