// libs/keyforge-compute/src/builder.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_adapter::conversion;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_core::ScoringSession;
use keyforge_model::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_model::CostMatrixSource;
use keyforge_protocol::JobRequest;
use keyforge_physics::ScoringEngine;
use keyforge_model::keycodes::KeycodeRegistry;
use std::sync::Arc;
use tracing::info;

/// A builder for constructing `ScoringSession` instances from various asset sources.
pub struct SessionBuilder<'a> {
    loader: &'a dyn AssetLoader,
}

impl<'a> SessionBuilder<'a> {
    /// Creates a new `SessionBuilder` using the provided asset loader.
    pub fn new(loader: &'a dyn AssetLoader) -> Self {
        Self { loader }
    }

    /// Builds a new scoring session by loading all necessary assets from the loader.
    ///
    /// This method is async as it may involve I/O or network requests to fetch 
    /// keyboards, corpora, or cost matrices.
    #[allow(clippy::too_many_arguments)]
    pub async fn build(
        &self,
        keyboard_name: &str,
        corpora: &[CorpusSource],
        weights: &ScoringWeights,
        params: &SearchParams,
        keycodes_filename: &str,
        cost_matrix: &CostMatrixSource,
        seed: Option<u64>,
    ) -> LoaderResult<ScoringSession> {
        // 1. Load Assets
        let kb_def = self.loader.load_keyboard(keyboard_name).await?;
        
        let domain_corpora: Vec<CorpusSource> = corpora
            .iter()
            .map(conversion::to_domain_corpus_source)
            .collect();
            
        let corpus = self.loader.load_corpus(&domain_corpora).await?;
        let registry = self.loader.load_keycodes(keycodes_filename).await
            .unwrap_or_else(|_| KeycodeRegistry::new_with_defaults());

        let raw_costs = match cost_matrix {
            CostMatrixSource::Predefined(name) => self.loader.load_cost_matrix(name).await?,
            CostMatrixSource::Custom(json) => serde_json::from_str(json)
                .map_err(|e| keyforge_model::error::ForgeError::InvalidData(format!("Invalid custom cost JSON: {}", e)))?,
        };

        // 2. Convert to Domain types
        let domain_kb = keyforge_model::Keyboard::new(
            kb_def.geometry.keys.clone(),
            kb_def.geometry.home_row
        ).map_err(|e| keyforge_model::error::ForgeError::InvalidData(format!("Invalid keyboard definition: {}", e)))?;

        let domain_rubric = conversion::to_domain_rubric(weights);
        let domain_config = conversion::to_domain_config(params, seed.unwrap_or(42));
        
        let overrides = raw_costs.resolve(&kb_def.geometry);

        // 3. Compile Engine
        let engine = ScoringEngine::new(&domain_kb, &corpus, &domain_rubric, &overrides)?;

        // TELEMETRY: Log engine stats
        let keys = engine.key_count();
        let trigrams = engine.trigram_count();
        // Rough heuristic: 1M ops/sec baseline, scales linearly with trigrams
        // Base cost ~ 50ns per bigram. Trigrams add ~10ns each.
        // This is just for log estimation.
        let est_ops = if trigrams > 0 { 50_000_000 / trigrams } else { 10_000_000 };
        
        info!(
            "compiled_engine keys={} trigrams={} est_ops_per_sec={}", 
            keys, trigrams, est_ops
        );

        Ok(ScoringSession {
            engine: Arc::new(engine),
            registry: Arc::new(registry),
            search_config: domain_config,
        })
    }

    /// Builds a scoring session directly from a `JobRequest` DTO.
    pub async fn build_from_job(&self, job: &JobRequest) -> LoaderResult<ScoringSession> {
        self.build_preloaded(
            &job.definition,
            &job.corpora,
            &job.weights,
            &job.params,
            "keycodes.json",
            &job.cost_matrix,
            None
        ).await
    }

    /// Builds a scoring session using a pre-loaded keyboard definition.
    ///
    /// This is useful when the keyboard geometry is already known (e.g., in a 
    /// distributed compute node after receiving a job).
    #[allow(clippy::too_many_arguments)]
    pub async fn build_preloaded(
        &self,
        kb_def: &keyforge_model::geometry::KeyboardDefinition,
        corpora: &[CorpusSource],
        weights: &ScoringWeights,
        params: &SearchParams,
        keycodes_filename: &str,
        cost_matrix: &CostMatrixSource,
        seed: Option<u64>,
    ) -> LoaderResult<ScoringSession> {
        // 1. Load Assets (Only those not preloaded)
        let domain_corpora: Vec<CorpusSource> = corpora
            .iter()
            .map(conversion::to_domain_corpus_source)
            .collect();

        let corpus = self.loader.load_corpus(&domain_corpora).await?;
        let registry = self.loader.load_keycodes(keycodes_filename).await
            .unwrap_or_else(|_| KeycodeRegistry::new_with_defaults());

        let raw_costs = match cost_matrix {
            CostMatrixSource::Predefined(name) => self.loader.load_cost_matrix(name).await?,
            CostMatrixSource::Custom(json) => serde_json::from_str(json)
                .map_err(|e| keyforge_model::error::ForgeError::InvalidData(format!("Invalid custom cost JSON: {}", e)))?,
        };

        // 2. Convert to Domain types
        let domain_kb = conversion::to_domain_keyboard(&kb_def.geometry);
        let domain_rubric = conversion::to_domain_rubric(weights);
        let domain_config = conversion::to_domain_config(params, seed.unwrap_or(42));
        
        let overrides = raw_costs.resolve(&kb_def.geometry);

        // 3. Compile Engine
        let engine = ScoringEngine::new(&domain_kb, &corpus, &domain_rubric, &overrides)?;

        // TELEMETRY: Log engine stats
        let keys = engine.key_count();
        let trigrams = engine.trigram_count();
        let est_ops = if trigrams > 0 { 50_000_000 / trigrams } else { 10_000_000 };
        
        info!(
            "compiled_engine keys={} trigrams={} est_ops_per_sec={}", 
            keys, trigrams, est_ops
        );

        Ok(ScoringSession {
            engine: Arc::new(engine),
            registry: Arc::new(registry),
            search_config: domain_config,
        })
    }
}