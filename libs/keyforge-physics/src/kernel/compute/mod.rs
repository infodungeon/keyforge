pub mod state;
pub mod flow;
pub mod scoring;
pub mod delta;
pub mod analysis;
#[cfg(test)]
pub mod tests;

// Re-export public items
pub use state::PhysicsScratch;
pub use scoring::score_layout;
pub use analysis::analyze_layout;

// Re-export crate-internal items
pub(crate) use state::PosMap;
pub(crate) use delta::calculate_swap_delta;
