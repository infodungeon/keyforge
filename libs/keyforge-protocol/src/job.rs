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
                ..Default::default()
            },
            keyboard: source.definition.meta.name.clone(),
            corpora: source.to_domain_corpus_sources(),
            cost_matrix: source.to_domain_cost_matrix(),
            seed: None,
            search: source.to_domain_params(),
            weights: source.to_domain_weights(),
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
    pub definition: KeyboardDefinitionDto,
    pub weights: ScoringWeightsDto,
    pub params: SearchParamsDto,
    pub pinned_keys: LimitedVec<KeyConstraintDto>,
    #[serde(default = "default_corpora")]
    pub corpora: LimitedVec<CorpusSourceDto>,
    #[serde(default = "default_cost_matrix")]
    pub cost_matrix: CostMatrixSourceDto,
    pub biometrics: LimitedVec<BiometricSample>,
    #[serde(default)]
    pub parent_job_id: Option<String>,
    #[serde(default)]
    pub baseline_score: Option<f32>,
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

    #[must_use]
    pub fn to_domain_weights(&self) -> keyforge_model::config::ScoringWeights {
        keyforge_model::config::ScoringWeights {
            weights: self.weights.weights.clone(),
            finger_penalty_scale: self.weights.finger_penalty_scale,
            comfortable_scissors: self.weights.comfortable_scissors.clone(),
        }
    }

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

    #[must_use]
    pub fn to_domain_cost_matrix(&self) -> keyforge_model::config::CostMatrixSource {
        match &self.cost_matrix {
            CostMatrixSourceDto::Predefined(s) => {
                keyforge_model::config::CostMatrixSource::Predefined(s.clone())
            }
        }
    }

    #[must_use]
    pub fn to_domain_geometry(&self) -> keyforge_model::geometry::KeyboardGeometry {
        keyforge_model::geometry::KeyboardGeometry {
            keys: self
                .definition
                .geometry
                .keys
                .iter()
                .map(|k| keyforge_model::geometry::KeyNode {
                    index: k.index as usize,
                    label: k.label.clone(),
                    x: k.x,
                    y: k.y,
                    w: k.w,
                    h: k.h,
                    hand: k.hand.into(),
                    finger: k.finger.into(),
                    row: k.row.into(),
                    col: k.col.into(),
                    is_home: k.is_home,
                    is_stretch: k.is_stretch,
                    r: k.r,
                    rx: k.rx,
                    ry: k.ry,
                })
                .collect(),
            prime_slots: self
                .definition
                .geometry
                .prime_slots
                .iter()
                .map(|&i| i.into())
                .collect(),
            med_slots: self
                .definition
                .geometry
                .med_slots
                .iter()
                .map(|&i| i.into())
                .collect(),
            low_slots: self
                .definition
                .geometry
                .low_slots
                .iter()
                .map(|&i| i.into())
                .collect(),
            home_row: self.definition.geometry.home_row,
        }
    }

    /// Generates a unique identifier for the job configuration.
    ///
    /// # Errors
    ///
    /// Returns `ModelError` if the configuration parts cannot be hashed or are invalid.
    pub fn id(&self) -> Result<String, keyforge_model::error::ModelError> {
        let corpora_fingerprint =
            keyforge_model::job::calculate_corpora_fingerprint(&self.to_domain_corpus_sources());

        keyforge_model::job::JobIdentifier::from_parts(
            &self.to_domain_geometry(),
            &self.to_domain_weights(),
            &self.to_domain_params(),
            &self.to_domain_pinned_keys(),
            &corpora_fingerprint,
            &self.to_domain_cost_matrix(),
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

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobRequest {
    #[serde(default = "default_version")]
    pub version: u32,
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

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobResponse {
    pub job_id: String,
    pub is_new: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobQueueResponse {
    pub job_id: Option<String>,
    pub config: Option<JobConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct ResultSubmission {
    #[serde(default = "default_version")]
    pub version: u32,
    pub job_id: String,
    pub layout: String,
    pub score: f32,
    #[serde(default)]
    pub raw_score: i64,
    pub timestamp: u64,
    pub nonce: u64,
    pub node_id: String,
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

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct JobDetailedStatus {
    pub job_id: String,
    pub status: JobStatusDto,
    pub best_score: Option<f32>,
    pub best_layout: Option<String>,
    pub total_samples: usize,
}
