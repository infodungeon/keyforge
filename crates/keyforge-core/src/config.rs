// Re-export protocol types
pub use crate::protocol::config::*;

use std::fs;
use std::path::Path;

// Define Trait
pub trait ConfigLoader {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Self;
}

// Implement Trait for the Protocol Type
impl ConfigLoader for ScoringWeights {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path).expect("Failed to read weights");
        serde_json::from_str(&content).expect("Failed to parse weights")
    }
}
