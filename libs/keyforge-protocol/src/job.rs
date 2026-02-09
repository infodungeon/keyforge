// libs/keyforge-protocol/src/job.rs

use crate::assets::BiometricSample;
use crate::config::{
    CorpusSourceDto, CostMatrixSourceDto, KeyConstraintDto, KeyboardDefinitionDto,
    ScoringWeightsDto, SearchParamsDto,
};
use crate::types::{JobStatusDto, LimitedVec};
use crate::PROTOCOL_VERSION;
use keyforge_model::{LayoutValidator, Projection, Validator};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

impl Projection<JobConfig> for keyforge_model::config::Config {
    fn project(source: JobConfig) -> Result<Self, keyforge_model::error::ForgeError> {
        Ok(Self {
            meta: keyforge_model::ProjectMeta {
                name: source.definition.meta.name.clone(),
                author: source.definition.meta.author.clone(),
                version: source.definition.meta.version.clone(),
            },
            keyboard: source.definition.meta.name.clone(),
            corpora: source.to_domain_corpus_sources(),
            cost_matrix: source.to_domain_cost_matrix(),
            seed: None,
            search: source.to_domain_params(),
            weights: source
                .to_domain_weights()
                .map_err(keyforge_model::error::ForgeError::InvalidData)?,
            defs: keyforge_model::config::LayoutDefinitions::default(),
            engine: keyforge_model::config::EngineConfig::default(),
            pinned_keys: source.to_domain_pinned_keys(),
        })
    }
}

fn default_version() -> u32 {
    PROTOCOL_VERSION
}
fn default_cost_matrix() -> CostMatrixSourceDto {
    CostMatrixSourceDto::Predefined("cost_matrix.json".to_string())
}
fn default_corpora() -> LimitedVec<CorpusSourceDto> {
    LimitedVec(vec![CorpusSourceDto {
        id: "en_small.json".into(),
        weight: 1.0,
        hash: None,
    }])
}

/// Full configuration for a running job.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobConfig {
    /// Physical keyboard definition.
    pub definition: KeyboardDefinitionDto,
    /// Ergonomic weights and penalties.
    pub weights: ScoringWeightsDto,
    /// Search and optimization parameters.
    pub params: SearchParamsDto,
    /// List of keys pinned to specific indices.
    pub pinned_keys: LimitedVec<KeyConstraintDto>,
    /// Linguistic corpora to use for scoring.
    #[serde(default = "default_corpora")]
    pub corpora: LimitedVec<CorpusSourceDto>,
    /// Biomechanical cost model source.
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSourceDto,
    /// Collected user biometric samples for personalization.
    pub biometrics: LimitedVec<BiometricSample>,
    /// Optional ID of a parent job.
    #[serde(default)]
    pub parent_job_id: Option<String>,
    /// Optional baseline score for comparison.
    #[serde(default)]
    pub baseline_score: Option<f32>,
    /// List of parent job hashes for lineage tracking.
    pub parents: LimitedVec<String>,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            definition: keyforge_model::geometry::KeyboardDefinition::default().into(),
            weights: keyforge_model::config::ScoringWeights::default().into(),
            params: keyforge_model::config::SearchParams::default().into(),
            pinned_keys: LimitedVec(vec![]),
            corpora: default_corpora(),
            cost_matrix: default_cost_matrix(),
            biometrics: LimitedVec(vec![]),
            parent_job_id: None,
            baseline_score: None,
            parents: LimitedVec(vec![]),
        }
    }
}

impl JobConfig {
    /// Converts protocol corpora to domain model corpora.
    #[must_use]
    pub fn to_domain_corpus_sources(&self) -> Vec<keyforge_model::config::CorpusSource> {
        self.corpora
            .iter()
            .map(|c| keyforge_model::config::CorpusSource {
                id: c.id.clone(),
                weight: c.weight,
                hash: c.hash.clone(),
            })
            .collect()
    }

    /// Converts protocol weights to domain model weights.
    ///
    /// # Errors
    /// Returns an error if any weight value is invalid.
    pub fn to_domain_weights(&self) -> Result<keyforge_model::config::ScoringWeights, String> {
        let mut weights = std::collections::HashMap::new();
        for (k, &v) in &self.weights.weights {
            weights.insert(k.clone(), keyforge_model::types::Score::from_f32(v)?);
        }

        let mut finger_penalty_scale = [keyforge_model::types::Score::default(); 5];
        for (i, &v) in self.weights.finger_penalty_scale.iter().enumerate() {
            finger_penalty_scale[i] = keyforge_model::types::Score::from_f32(v)?;
        }

        Ok(keyforge_model::config::ScoringWeights {
            weights,
            finger_penalty_scale,
            comfortable_scissors: self.weights.comfortable_scissors.clone(),
        })
    }

    /// Converts protocol search params to domain model params.
    #[must_use]
    pub fn to_domain_params(&self) -> keyforge_model::config::SearchParams {
        keyforge_model::config::SearchParams {
            params: {
                let mut p = std::collections::HashMap::new();
                #[allow(clippy::cast_precision_loss)]
                p.insert("search_steps".into(), self.params.iterations as f32);
                if let Some(r) = self.params.reheats {
                    #[allow(clippy::cast_precision_loss)]
                    p.insert("reheats".into(), r as f32);
                }
                p
            },
            seed: self.params.seed,
            include_thumbs: self.params.include_thumbs,
        }
    }

    /// Converts protocol pinned keys to domain model constraints.
    #[must_use]
    pub fn to_domain_pinned_keys(&self) -> Vec<keyforge_model::config::KeyConstraint> {
        self.pinned_keys
            .iter()
            .map(|p| keyforge_model::config::KeyConstraint {
                index: p.index.into(),
                key: p.key.clone(),
            })
            .collect()
    }

    /// Converts protocol cost matrix to domain model source.
    #[must_use]
    pub fn to_domain_cost_matrix(&self) -> keyforge_model::config::CostMatrixSource {
        match &self.cost_matrix {
            CostMatrixSourceDto::Predefined(s) => {
                keyforge_model::config::CostMatrixSource::Predefined(s.clone())
            }
        }
    }

    /// Converts protocol geometry to domain model geometry.
    #[must_use]
    pub fn to_domain_geometry(&self) -> keyforge_model::geometry::KeyboardGeometry {
        let keys = self
            .definition
            .geometry
            .keys
            .iter()
            .map(|k| keyforge_model::geometry::KeyNode {
                index: k.index.into(),
                label: k.label.clone(),
                x: keyforge_model::types::SpatialUnit::from_f32(k.x),
                y: keyforge_model::types::SpatialUnit::from_f32(k.y),
                w: k.w,
                h: k.h,
                hand: k.hand.into(),
                finger: k.finger.into(),
                row: k.row.into(),
                col: k.col.into(),
                is_home: k.is_home,
                is_stretch: k.is_stretch,
                r: k.r,
                rx: keyforge_model::types::SpatialUnit::from_f32(k.rx),
                ry: keyforge_model::types::SpatialUnit::from_f32(k.ry),
            })
            .collect();

        let prime_slots = self
            .definition
            .geometry
            .prime_slots
            .iter()
            .map(|&i| i.into())
            .collect();
        let med_slots = self
            .definition
            .geometry
            .med_slots
            .iter()
            .map(|&i| i.into())
            .collect();
        let low_slots = self
            .definition
            .geometry
            .low_slots
            .iter()
            .map(|&i| i.into())
            .collect();

        keyforge_model::geometry::KeyboardGeometry::new(
            keys,
            prime_slots,
            med_slots,
            low_slots,
            keyforge_model::types::RowIndex::new(self.definition.geometry.home_row),
        )
    }

    /// Generates a unique identifier for the job configuration.
    ///
    /// # Errors
    ///
    /// Returns `ModelError` if the configuration parts cannot be hashed or are invalid.
    pub fn id(&self) -> Result<String, keyforge_model::error::ModelError> {
        let corpora_hash =
            keyforge_model::job::calculate_corpora_hash(&self.to_domain_corpus_sources());

        keyforge_model::job::JobIdentifier::from_parts(
            &self.to_domain_geometry(),
            &self
                .to_domain_weights()
                .map_err(keyforge_model::error::ModelError::Invariant)?,
            &self.to_domain_params(),
            &self.to_domain_pinned_keys(),
            &corpora_hash,
            &self.to_domain_cost_matrix(),
            None,
        )
        .map(|ident| ident.hash)
    }
}

impl Validator for JobConfig {
    fn validate(&self) -> Result<(), String> {
        self.weights.validate()?;
        self.params.validate()?;
        self.definition.validate()?;
        for (i, corpus) in self.corpora.iter().enumerate() {
            corpus.validate().map_err(|e| format!("Corpus #{i}: {e}"))?;
        }
        Ok(())
    }
}

/// A request to register a new job.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobRequest {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Job configuration.
    pub config: JobConfig,
}

impl Default for JobRequest {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            config: JobConfig::default(),
        }
    }
}

impl Validator for JobRequest {
    fn validate(&self) -> Result<(), String> {
        self.config.validate()
    }
}

/// Response from job registration.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobResponse {
    /// Unique ID of the job.
    pub job_id: String,
    /// True if the job was newly created.
    pub is_new: bool,
}

/// Response containing a job for a worker.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobQueueResponse {
    /// Unique ID of the job.
    pub job_id: Option<String>,
    /// Configuration for the worker.
    pub config: Option<JobConfig>,
}

/// A result submission from a compute node.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ResultSubmission {
    /// Protocol version.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Target job ID.
    pub job_id: String,
    /// Space-separated layout string.
    pub layout: String,
    /// Achieved score (f32).
    pub score: f32,
    /// Raw scaled score (i64).
    #[serde(default)]
    pub raw_score: i64,
    /// Submission timestamp.
    pub timestamp: u64,
    /// Security nonce.
    pub nonce: u64,
    /// ID of the node that produced this result.
    pub node_id: String,
    /// Cryptographic signature of the result.
    pub signature: String,
}

impl Validator for ResultSubmission {
    fn validate(&self) -> Result<(), String> {
        if self.job_id.trim().is_empty() {
            return Err("job_id cannot be empty".into());
        }
        LayoutValidator::validate_structure(&self.layout)?;
        Ok(())
    }
}

/// Detailed status of a job including best result.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobDetailedStatus {
    /// Job ID.
    pub job_id: String,
    /// Current lifecycle status.
    pub status: JobStatusDto,
    /// Best score found so far.
    pub best_score: Option<f32>,
    /// Best layout string found so far.
    pub best_layout: Option<String>,
    /// Total number of samples processed.
    pub total_samples: usize,
}
