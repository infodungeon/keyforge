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
