pub mod crossover;
pub mod mutation;
pub mod runner;

// NEW: Replica is now a module, not a single file
pub mod replica;

pub use self::replica::Replica;
pub use self::runner::{OptimizationOptions, Optimizer, ProgressCallback};
