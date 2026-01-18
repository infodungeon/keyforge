// libs/keyforge-model/src/config/source.rs

use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use utoipa::ToSchema;
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Filename for the default Cost Matrix asset.
pub const ASSET_COST_MATRIX: &str = "cost_matrix";

/// Definitions for layout tiers and critical bigrams.
/// Default weight for a corpus source.
pub const DEFAULT_CORPUS_WEIGHT: f32 = 1.0;

/// Defines a source for text corpus data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct CorpusSource {
    /// The identifier or path of the corpus.
    pub id: String,
    /// The weight multiplier for this corpus.
    pub weight: f32,
    /// Optional hash for integrity verification.
    #[serde(default, skip_serializing_if = "crate::utils::is_none")]
    #[cfg_attr(feature = "ts_bindings", ts(optional))]
    pub hash: Option<String>,
}

impl Validator for CorpusSource {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Corpus ID cannot be empty".to_string());
        }
        if self.weight <= 0.0 || !self.weight.is_finite() {
            return Err(format!("Invalid weight for corpus '{}': {}", self.id, self.weight));
        }
        Ok(())
    }
}

impl Hash for CorpusSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.weight.to_bits().hash(state);
        self.hash.hash(state);
    }
}

impl Default for CorpusSource {
    fn default() -> Self {
        Self {
            id: "text/en_std".to_string(),
            weight: DEFAULT_CORPUS_WEIGHT,
            hash: None,
        }
    }
}

impl FromStr for CorpusSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((id, weight_str)) = s.split_once(':') {
            let weight = weight_str
                .parse::<f32>()
                .map_err(|_| format!("invalid weight '{weight_str}' for corpus '{id}'"))?;

            if weight.is_nan() || weight <= f32::EPSILON {
                return Err(format!(
                    "weight for corpus '{id}' must be positive (got {weight})"
                ));
            }

            Ok(CorpusSource {
                id: id.trim().to_string(),
                weight,
                hash: None,
            })
        } else {
            Ok(CorpusSource {
                id: s.trim().to_string(),
                weight: DEFAULT_CORPUS_WEIGHT,
                hash: None,
            })
        }
    }
}

/// Source for the cost matrix data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub enum CostMatrixSource {
    /// A predefined cost matrix file (e.g. "`default_costmatrix.json`").
    Predefined(String),
}

impl Default for CostMatrixSource {
    fn default() -> Self { CostMatrixSource::Predefined(ASSET_COST_MATRIX.to_string()) }
}

impl fmt::Display for CostMatrixSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostMatrixSource::Predefined(s) => write!(f, "{s}"),
        }
    }
}
