// Copyright (c) 2025 KeyForge Contributors
//
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
use keyforge_adapter::conversion;
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_core::ScoringSession;
use keyforge_model::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_model::CostMatrixSource;
use keyforge_protocol::JobRequest;
use keyforge_physics::ScoringEngine;
use keyforge_model::keycodes::KeycodeRegistry;
use std::sync::Arc;

pub struct SessionBuilder<'a> {
    loader: &'a dyn AssetLoader,
}

impl<'a> SessionBuilder<'a> {
    pub fn new(loader: &'a dyn AssetLoader) -> Self {
        Self { loader }
    }

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
        // kb_def is loaded from disk -> Model type
        let kb_def = self.loader.load_keyboard(keyboard_name).await?;
        
        // Convert corpora DTO -> Model
        let domain_corpora: Vec<keyforge_model::config::CorpusSource> = corpora
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
        
        // kb_def is already Model. Construct the graph.
        let domain_kb = keyforge_model::Keyboard::new(
            kb_def.geometry.keys.clone(),
            kb_def.geometry.home_row
        ).map_err(|e| keyforge_model::error::ForgeError::InvalidData(format!("Invalid keyboard definition: {}", e)))?;

        let domain_rubric = conversion::to_domain_rubric(weights);
        let domain_config = conversion::to_domain_config(params, seed.unwrap_or(42));
        
        // Resolve costs using the Model geometry
        let overrides = raw_costs.resolve(&kb_def.geometry);

        // 3. Compile Engine
        let engine = ScoringEngine::new(&domain_kb, &corpus, &domain_rubric, &overrides)?;

        Ok(ScoringSession {
            engine: Arc::new(engine),
            registry: Arc::new(registry),
            search_config: domain_config,
        })
    }

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
        let domain_corpora: Vec<keyforge_model::config::CorpusSource> = corpora
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
        // kb_def is Model. Convert to Runtime Keyboard.
        let domain_kb = conversion::to_domain_keyboard(&kb_def.geometry);
        let domain_rubric = conversion::to_domain_rubric(weights);
        let domain_config = conversion::to_domain_config(params, seed.unwrap_or(42));
        
        // Resolve uses Model geometry.
        let overrides = raw_costs.resolve(&kb_def.geometry);

        // 3. Compile Engine
        let engine = ScoringEngine::new(&domain_kb, &corpus, &domain_rubric, &overrides)?;

        Ok(ScoringSession {
            engine: Arc::new(engine),
            registry: Arc::new(registry),
            search_config: domain_config,
        })
    }
}
