// libs/keyforge-compute/src/use_cases.rs

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

//! Unified Use Case implementations shared across components.

use crate::builder::SessionBuilder;
use crate::session::ScoringSession;
use keyforge_adapter::loader::AssetLoader;
use keyforge_model::job::JobIdentifier;
use keyforge_protocol::JobRequest;

/// Orchestrates the preparation of an optimization job.
#[derive(Debug)]
pub struct OptimizationUseCase;

impl OptimizationUseCase {
    /// Prepares a `ScoringSession` and `JobIdentifier` from a high-level request.
    ///
    /// This is the "Truth" implementation used by both Hive (Cloud) and CLI (Local).
    ///
    /// # Errors
    /// Returns `ForgeError` if asset loading or session building fails.
    pub async fn prepare_session<L: AssetLoader>(
        loader: &L,
        req: &JobRequest,
    ) -> Result<(JobIdentifier, ScoringSession), keyforge_model::error::ForgeError> {
        // 1. Identify the job (Deterministic Hash)
        let geometry = req.config.to_domain_geometry();
        let weights = req.config.to_domain_weights();
        let params = req.config.to_domain_params();
        let pinned = req.config.to_domain_pinned_keys();
        let mut corpora = req.config.to_domain_corpus_sources();
        let cost_matrix = req.config.to_domain_cost_matrix();

        // 1.5. Resolve hashes for content-addressable Job ID
        for src in &mut corpora {
            if src.hash.is_none() {
                if let Ok(h) = loader.get_hash(keyforge_model::AssetCategory::Corpus, &src.id).await {
                    src.hash = Some(h);
                }
            }
        }

        let mut cost_matrix_hash = None;
        let keyforge_model::CostMatrixSource::Predefined(ref name) = cost_matrix;
        if let Ok(h) = loader.get_hash(keyforge_model::AssetCategory::CostModel, name).await {
            cost_matrix_hash = Some(h);
        }

        let corpora_hash = keyforge_model::job::calculate_corpora_hash(&corpora);

        let id = JobIdentifier::try_from_parts(
            &geometry,
            &weights,
            &params,
            &pinned,
            &corpora_hash,
            &cost_matrix,
            cost_matrix_hash.as_deref(),
        )
        .map_err(|e| keyforge_model::error::ForgeError::Validation(e.to_string()))?;

        // 2. Build the session
        let kb_def = loader
            .load::<keyforge_model::geometry::KeyboardDefinition>(&req.config.definition.meta.name)
            .await
            .map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!(
                    "Failed to load keyboard {}: {}",
                    req.config.definition.meta.name, e
                ))
            })?;

        let keyforge_protocol::config::CostMatrixSourceDto::Predefined(cost_model_name) =
            &req.config.cost_matrix;

        let cost_model = loader
            .load::<keyforge_model::CostModel>(cost_model_name)
            .await
            .map_err(|e| {
                keyforge_model::error::ForgeError::Io(format!(
                    "Failed to load cost model {cost_model_name}: {e}"
                ))
            })?;

        let session = SessionBuilder::new(loader)
            .with_keyboard_def(kb_def)
            .with_corpus_obj(
                loader
                    .load_corpus(&corpora)
                    .await
                    .map_err(|e| keyforge_model::error::ForgeError::Io(e.to_string()))?,
            )
            .with_cost_model_obj(cost_model)
            .with_rubric(keyforge_adapter::conversion::to_domain_rubric(&weights))
            .with_config(keyforge_model::SearchConfig::Annealing {
                steps: params.get_search_steps(),
                start_temp: 10.0,
                end_temp: 0.1,
                seed: params.seed.unwrap_or(42),
                patience: 100,
                reheats: params.get_reheats(),
                reheat_factor: 0.5,
                include_thumbs: params.include_thumbs,
            })
            .with_biometrics(req.config.biometrics.to_vec())
            .build()
            .map_err(|e| keyforge_model::error::ForgeError::Config(e.to_string()))?;

        Ok((id, session))
    }
}
