pub mod api;
pub mod biometrics;
pub mod consts;
pub mod core_types;
pub mod corpus;
pub mod error;
pub mod geometry;
pub mod keycodes;
pub mod layouts;
pub mod optimizer;
pub mod scorer;
pub mod util;
pub mod verifier;

// CHANGED: Use local config module (which re-exports protocol types + adds loader trait)
pub mod config;

// Re-export protocol types
pub use keyforge_protocol as protocol;
// pub use keyforge_protocol::config; // <-- Removed, handled by mod config above
pub use keyforge_protocol::job;
