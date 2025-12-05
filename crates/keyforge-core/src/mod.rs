// ===== keyforge/crates/keyforge-core/src/optimizer/mod.rs =====
pub mod crossover;
pub mod initialization;
pub mod mutation; // Ensure this is pub mod
pub mod replica;
pub mod runner;

// Re-export specific structs for easier access
pub use self::replica::Replica;
pub use self::runner::{OptimizationOptions, OptimizationResult, Optimizer, ProgressCallback};