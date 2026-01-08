// libs/keyforge-persistence/src/project.rs

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

use keyforge_model::{
    config::{CorpusSource, ScoringWeights, SearchParams},
    CostMatrixSource, KeyConstraint,
};
use serde::{Deserialize, Serialize};

/// Metadata about a user project.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A persistable definition of an optimization experiment.
/// Contains all "ingredients" needed to compile a Runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Metadata about the project.
    #[serde(default)]
    pub meta: ProjectMeta,

    /// Name or Path of the keyboard definition (e.g. "corne", "ansi_104")
    pub keyboard: String,

    /// List of corpora to blend
    pub corpora: Vec<CorpusSource>,

    /// Scoring parameters (weights for uncomfortable keys, etc.)
    pub weights: ScoringWeights,

    /// Search configuration (annealing steps, temperature, etc.)
    pub params: SearchParams,

    /// User-defined constraints (pinned keys)
    #[serde(default)]
    pub constraints: Vec<KeyConstraint>,

    /// Source for the cost matrix (biomechanical profile)
    #[serde(default)]
    pub cost_matrix: CostMatrixSource,

    /// Optional seed for deterministic reproducibility
    #[serde(default)]
    pub seed: Option<u64>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            meta: ProjectMeta::default(),
            keyboard: "ortho_30".to_string(),
            corpora: vec![CorpusSource::default()],
            weights: ScoringWeights::default(),
            params: SearchParams::default(),
            constraints: Vec::new(),
            cost_matrix: CostMatrixSource::default(),
            seed: None,
        }
    }
}
