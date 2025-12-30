use crate::config::{ScoringWeights, SearchParams};
use crate::geometry::KeyboardGeometry;
use crate::{CostMatrixSource, KeyConstraint};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use ts_rs::TS;

#[derive(Debug, thiserror::Error, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub enum JobIdError {
    #[error("Serialization error: {0}")]
    Serialize(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct JobIdentifier {
    pub hash: String,
}

impl JobIdentifier {
    pub fn try_from_parts(
        geometry: &KeyboardGeometry,
        weights: &ScoringWeights,
        params: &SearchParams,
        pinned_keys: &[KeyConstraint],
        corpus_fingerprint: &str,
        cost_matrix: &CostMatrixSource,
    ) -> Result<Self, JobIdError> {
        let mut hasher = Sha256::new();

        // 1. Geometry
        let geo_json = serde_json::to_string(geometry).map_err(|e| JobIdError::Serialize(e.to_string()))?;
        hasher.update(geo_json.as_bytes());
        hasher.update(b"|");

        // 2. Weights
        let w_json = serde_json::to_string(weights).map_err(|e| JobIdError::Serialize(e.to_string()))?;
        hasher.update(w_json.as_bytes());
        hasher.update(b"|");

        // 3. Params
        let p_json = serde_json::to_string(params).map_err(|e| JobIdError::Serialize(e.to_string()))?;
        hasher.update(p_json.as_bytes());
        hasher.update(b"|");

        // 4. Pinned Keys
        let pins_json = serde_json::to_string(pinned_keys).map_err(|e| JobIdError::Serialize(e.to_string()))?;
        hasher.update(pins_json.as_bytes());
        hasher.update(b"|");

        // 5. Corpus
        hasher.update(corpus_fingerprint.as_bytes());
        hasher.update(b"|");

        // 6. Cost Matrix
        let cost_json = serde_json::to_string(cost_matrix).map_err(|e| JobIdError::Serialize(e.to_string()))?;
        hasher.update(cost_json.as_bytes());

        let result = hasher.finalize();
        let hash = hex::encode(result);

        Ok(Self { hash })
    }
}
