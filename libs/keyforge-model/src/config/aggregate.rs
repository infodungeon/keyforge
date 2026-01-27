// libs/keyforge-model/src/config/aggregate.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::config::definitions::LayoutDefinitions;
use crate::config::search::{SearchConfig, SearchParams};
use crate::config::source::{CorpusSource, CostMatrixSource};
use crate::config::weights::ScoringWeights;
use crate::corpus::Corpus;
use crate::cost_model::CostModel;
use crate::keyboard::Keyboard;
use crate::layout::Layout;
use crate::rubric::Rubric;
use crate::types::KeyCode;
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Metadata about a user project or session.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]

pub struct ProjectMeta {
    /// The display name of the project.
    pub name: String,
    /// The version string for the project.
    pub version: String,
    /// The author of the project.
    #[serde(default)]
    pub author: String,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "0.1.0".to_string(),
            author: "Anonymous".to_string(),
        }
    }
}

/// The root configuration aggregate for a `KeyForge` session.
/// This structure is also used for persistence (Project files).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]

pub struct Config {
    /// Metadata about the configuration/project.
    #[serde(default)]
    pub meta: ProjectMeta,

    /// Name or Path of the keyboard definition (e.g. "corne", "`ansi_104`")
    pub keyboard: String,

    /// List of corpora to blend
    pub corpora: Vec<CorpusSource>,

    /// Source for the cost matrix (biomechanical profile)
    #[serde(default)]
    pub cost_matrix: CostMatrixSource,

    /// Optional seed for deterministic reproducibility
    #[serde(default)]
    pub seed: Option<u64>,

    /// Search parameters for the optimization engine.
    pub search: SearchParams,
    /// Hardware-specific engine parameters.
    #[serde(default)]
    pub engine: crate::config::EngineConfig,
    /// Weights for the physics scoring engine.
    pub weights: ScoringWeights,
    /// Definitions for layout tiers and critical bigrams.
    pub defs: LayoutDefinitions,
    /// Keys pinned to specific positions.
    #[serde(default)]
    pub pinned_keys: Vec<crate::config::constraints::KeyConstraint>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            meta: ProjectMeta::default(),
            keyboard: "ortho_30".to_string(),
            corpora: vec![CorpusSource::default()],
            cost_matrix: CostMatrixSource::default(),
            seed: None,
            search: SearchParams::default(),
            engine: crate::config::EngineConfig::default(),
            weights: ScoringWeights::default(),
            defs: LayoutDefinitions::default(),
            pinned_keys: Vec::new(),
        }
    }
}

impl Validator for Config {
    fn validate(&self) -> Result<(), String> {
        self.search.validate()?;
        self.engine.validate()?;
        self.weights.validate()?;
        self.defs.validate()?;
        if self.corpora.is_empty() {
            return Err("At least one corpus source is required".into());
        }
        for c in &self.corpora {
            c.validate()?;
        }
        for p in &self.pinned_keys {
            p.validate()?;
        }
        Ok(())
    }
}

/// A request structure for performing common engine operations.
#[derive(Clone, Debug)]
pub struct EngineRequest {
    /// The physical keyboard geometry.
    pub keyboard: Arc<Keyboard>,
    /// The language statistics to use.
    pub corpus: Arc<Corpus>,
    /// The ergonomic weights to apply.
    pub rubric: Arc<Rubric>,
    /// The cost model to use.
    pub cost_model: Arc<CostModel>,
    /// Optimization and search parameters.
    pub config: SearchConfig,
    /// Engine hardware optimization parameters.
    pub engine_config: crate::config::EngineConfig,
    /// The starting layout for the operation.
    pub initial_layout: Option<Layout>,
    /// Keys that must remain in their initial positions.
    pub pinned_keys: Vec<Option<KeyCode>>,
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_config_aggregate_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Trigger failure in one of the children
        config.search.params.insert("search_epochs".into(), 0.0);
        assert!(config.validate().is_err());
    }
}
