pub mod agent; // The Core Agent Logic
pub mod hw_detect;
pub mod logging;
pub mod models;
pub mod nice; // NEW

// Re-export the main runner
pub use agent::run_worker;
