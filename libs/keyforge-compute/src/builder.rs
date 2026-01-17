// libs/keyforge-compute/src/builder.rs

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

use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_core::ScoringSession;
use keyforge_model::config::{CorpusSource, CostMatrixSource};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{Corpus, Rubric, SearchConfig, CostModel};
use keyforge_physics::ScoringEngine;
use std::sync::Arc;
use std::fmt;

/// Builder for constructing a `ScoringSession` (Runtime).
pub struct SessionBuilder<'a> {
    loader: &'a dyn AssetLoader,
    keyboard: Option<Arc<KeyboardDefinition>>,
    corpus: Option<Arc<Corpus>>,
    rubric: Option<Arc<Rubric>>,
    cost_model: Option<Arc<CostModel>>,
    registry: Option<Arc<KeycodeRegistry>>,
    search_config: Option<SearchConfig>,
}

impl<'a> fmt::Debug for SessionBuilder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionBuilder")
            .field("keyboard", &self.keyboard)
            .field("corpus", &self.corpus)
            .field("rubric", &self.rubric)
            .field("cost_model", &self.cost_model)
            .field("registry", &self.registry)
            .field("search_config", &self.search_config)
            .finish()
    }
}

impl<'a> SessionBuilder<'a> {
    pub fn new(loader: &'a dyn AssetLoader) -> Self {
        Self {
            loader,
            keyboard: None,
            corpus: None,
            rubric: None,
            cost_model: None,
            registry: None,
            search_config: None,
        }
    }

    pub async fn with_keyboard(mut self, name: &str) -> LoaderResult<Self> {
        self.keyboard = Some(self.loader.load_keyboard(name).await?);
        Ok(self)
    }
    
    pub fn with_keyboard_def(mut self, def: Arc<KeyboardDefinition>) -> Self {
        self.keyboard = Some(def);
        self
    }

    pub async fn with_corpus(mut self, sources: &[CorpusSource]) -> LoaderResult<Self> {
        self.corpus = Some(self.loader.load_corpus(sources).await?);
        Ok(self)
    }
    
    pub fn with_corpus_obj(mut self, corpus: Arc<Corpus>) -> Self {
        self.corpus = Some(corpus);
        self
    }

    pub async fn with_cost_matrix(mut self, source: &CostMatrixSource) -> LoaderResult<Self> {
        match source {
            CostMatrixSource::Predefined(name) => {
                self.cost_model = Some(self.loader.load_cost_model(name).await?);
            }
        }
        Ok(self)
    }
    
    pub fn with_cost_model_obj(mut self, model: Arc<CostModel>) -> Self {
        self.cost_model = Some(model);
        self
    }

    pub async fn with_keycodes(mut self, name: &str) -> LoaderResult<Self> {
        self.registry = Some(self.loader.load_keycodes(name).await?);
        Ok(self)
    }

    pub fn with_rubric(mut self, rubric: Rubric) -> Self {
        self.rubric = Some(Arc::new(rubric));
        self
    }

    pub fn with_config(mut self, config: SearchConfig) -> Self {
        self.search_config = Some(config);
        self
    }

    pub fn build(self) -> LoaderResult<ScoringSession> {
        let kb_def = self.keyboard.ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing keyboard".into()))?;
        let corpus = self.corpus.ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing corpus".into()))?;
        let rubric = self.rubric.unwrap_or_else(|| Arc::new(Rubric::default()));
        let cost_model = self.cost_model.ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing cost model".into()))?;
        let registry = self.registry.unwrap_or_else(|| Arc::new(KeycodeRegistry::default()));
        let config = self.search_config.unwrap_or_default();

        // Create Keyboard from Definition (using home_row from geometry)
        let keyboard = Arc::new(keyforge_model::Keyboard::new(
            kb_def.geometry.keys.clone(),
            kb_def.geometry.home_row,
        ).map_err(|e| keyforge_model::error::ForgeError::InvalidData(e.to_string()))?);

        let engine = ScoringEngine::new(&keyboard, &corpus, &rubric, &cost_model)?;

        Ok(ScoringSession::new(Arc::new(engine), registry, config))
    }
}
