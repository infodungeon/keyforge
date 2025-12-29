use crate::config::{CostMatrixSource, KeyConstraint, ScoringWeights, SearchParams};
use crate::geometry::KeyboardGeometry;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct JobIdentifier {
    pub hash: String,
}

#[derive(Debug, Error)]
pub enum JobIdError {
    #[error("Failed to serialize job identifier component: {0}")]
    Serialize(String),
}

impl JobIdentifier {
    pub fn try_from_parts(
        geometry: &KeyboardGeometry,
        weights: &ScoringWeights,
        params: &SearchParams,
        pinned_keys: &[KeyConstraint],
        corpus_name: &str,
        cost_matrix: &CostMatrixSource,
    ) -> Result<Self, JobIdError> {
        let mut hasher = Sha256::new();

        fn feed<T: serde::Serialize>(hasher: &mut Sha256, value: &T) -> Result<(), JobIdError> {
            let bytes =
                bincode::serialize(value).map_err(|e| JobIdError::Serialize(e.to_string()))?;
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
            Ok(())
        }

        feed(&mut hasher, geometry)?;
        feed(&mut hasher, weights)?;
        feed(&mut hasher, params)?;

        // Sort pins for canonicalization
        let mut sorted_pins = pinned_keys.to_vec();
        sorted_pins.sort_by(|a, b| a.index.cmp(&b.index));
        feed(&mut hasher, &sorted_pins)?;

        hasher.update((corpus_name.len() as u64).to_le_bytes());
        hasher.update(corpus_name.as_bytes());

        match cost_matrix {
            CostMatrixSource::Predefined(s) => {
                hasher.update(b"PRE");
                hasher.update(s.as_bytes());
            }
            CostMatrixSource::Custom(s) => {
                hasher.update(b"CUST");
                hasher.update(s.as_bytes());
            }
        }

        let result = hasher.finalize();
        Ok(Self {
            hash: hex::encode(result),
        })
    }

    pub fn from_parts(
        geometry: &KeyboardGeometry,
        weights: &ScoringWeights,
        params: &SearchParams,
        pinned_keys: &[KeyConstraint],
        corpus_name: &str,
        cost_matrix: &CostMatrixSource,
    ) -> Self {
        Self::try_from_parts(
            geometry,
            weights,
            params,
            pinned_keys,
            corpus_name,
            cost_matrix,
        )
        .expect("JobIdentifier::from_parts failed")
    }
}
