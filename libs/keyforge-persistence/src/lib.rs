// Copyright (c) 2025 KeyForge Contributors
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
