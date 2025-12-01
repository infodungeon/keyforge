use crate::config::ScoringWeights; // Removed Config
use crate::geometry::KeyboardGeometry;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct JobIdentifier {
    pub hash: String,
}

impl JobIdentifier {
    /// Generates a deterministic hash based on the inputs that define the search landscape.
    pub fn from_parts(
        geometry: &KeyboardGeometry,
        weights: &ScoringWeights,
        pinned_keys: &str,
        corpus_name: &str, // e.g. "english_1M"
    ) -> Self {
        let mut hasher = Sha256::new();

        // 1. Geometry (Keys define the physics)
        // We serialize to JSON to ensure structural consistency
        let geo_json = serde_json::to_string(&geometry.keys).unwrap();
        hasher.update(geo_json.as_bytes());

        // 2. Weights (Define the objective function)
        let weights_json = serde_json::to_string(weights).unwrap();
        hasher.update(weights_json.as_bytes());

        // 3. Constraints
        hasher.update(pinned_keys.as_bytes());

        // 4. Data Source
        hasher.update(corpus_name.as_bytes());

        let result = hasher.finalize();
        Self {
            hash: hex::encode(result),
        }
    }
}
