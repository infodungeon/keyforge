use crate::config::{ScoringWeights, SearchParams};
use crate::geometry::KeyboardGeometry;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct JobIdentifier {
    pub hash: String,
}

impl JobIdentifier {
    pub fn from_parts(
        geometry: &KeyboardGeometry,
        weights: &ScoringWeights,
        params: &SearchParams,
        pinned_keys: &str,
        corpus_name: &str,
        cost_matrix: &str,
    ) -> Self {
        let mut hasher = Sha256::new();

        let geo_json = serde_json::to_string(&geometry.keys).unwrap();
        hasher.update(geo_json.as_bytes());

        let weights_json = serde_json::to_string(weights).unwrap();
        hasher.update(weights_json.as_bytes());

        let params_json = serde_json::to_string(params).unwrap();
        hasher.update(params_json.as_bytes());

        hasher.update(pinned_keys.as_bytes());
        hasher.update(corpus_name.as_bytes());
        hasher.update(cost_matrix.as_bytes());

        let result = hasher.finalize();
        Self {
            hash: hex::encode(result),
        }
    }
}
