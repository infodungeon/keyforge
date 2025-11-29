pub mod api;
pub mod config;
pub mod geometry;
pub mod layouts;
pub mod optimizer;
pub mod scorer;
// cmd and reports are binary modules (in main.rs or distinct files),
// but if you want to test them, they might need to be pub here.
// For this refactor, they are modules of the binary crate (main).
