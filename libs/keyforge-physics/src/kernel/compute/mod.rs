pub mod analysis;
pub mod delta;
pub mod flow;
pub mod scoring;
pub mod state;
#[cfg(test)]
pub mod tests;

// Re-export public items
pub use analysis::analyze_layout;
pub use scoring::score_layout;
pub use state::PhysicsScratch;

// Re-export crate-internal items
pub(crate) use delta::calculate_swap_delta;
pub(crate) use state::PosMap;
