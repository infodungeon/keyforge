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

use crate::biometrics::BiometricProfiler;
use crate::hardware::HardwareProbe;
use crate::session::ScoringSession;
use keyforge_model::config::{CorpusSource, CostMatrixSource};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::loader::{AssetLoader, LoaderResult};
use keyforge_model::{Corpus, CostModel, Rubric, SearchConfig};
use keyforge_protocol::BiometricSample;
use std::fmt;
use std::sync::Arc;

/// Builder for constructing a `ScoringSession` (Runtime).
pub struct SessionBuilder<'a, L: AssetLoader> {
    loader: &'a L,
    keyboard: Option<Arc<KeyboardDefinition>>,
    corpus: Option<Arc<Corpus>>,
    rubric: Option<Arc<Rubric>>,
    cost_model: Option<Arc<CostModel>>,
    registry: Option<Arc<KeycodeRegistry>>,
    search_config: Option<SearchConfig>,
    biometrics: Vec<BiometricSample>,
}

impl<L: AssetLoader> fmt::Debug for SessionBuilder<'_, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionBuilder")
            .field("keyboard", &self.keyboard)
            .field("corpus", &self.corpus)
            .field("rubric", &self.rubric)
            .field("cost_model", &self.cost_model)
            .field("registry", &self.registry)
            .field("search_config", &self.search_config)
            .field("biometrics_count", &self.biometrics.len())
            .finish()
    }
}

impl<'a, L: AssetLoader> SessionBuilder<'a, L> {
    pub fn new(loader: &'a L) -> Self {
        Self {
            loader,
            keyboard: None,
            corpus: None,
            rubric: None,
            cost_model: None,
            registry: None,
            search_config: None,
            biometrics: Vec::new(),
        }
    }

    /// Adds a keyboard definition to the session.
    ///
    /// # Errors
    /// Returns `LoaderResult` if the keyboard fails to load.
    pub async fn with_keyboard(mut self, name: &str) -> LoaderResult<Self> {
        self.keyboard = Some(self.loader.load::<KeyboardDefinition>(name).await?);
        Ok(self)
    }

    #[must_use]
    pub fn with_keyboard_def(mut self, def: Arc<KeyboardDefinition>) -> Self {
        self.keyboard = Some(def);
        self
    }

    /// Adds a corpus to the session.
    ///
    /// # Errors
    /// Returns `LoaderResult` if the corpus fails to load.
    pub async fn with_corpus(mut self, sources: &[CorpusSource]) -> LoaderResult<Self> {
        self.corpus = Some(self.loader.load_corpus(sources).await?);
        Ok(self)
    }

    #[must_use]
    pub fn with_corpus_obj(mut self, corpus: Arc<Corpus>) -> Self {
        self.corpus = Some(corpus);
        self
    }

    /// Adds a cost matrix to the session.
    ///
    /// # Errors
    /// Returns `LoaderResult` if the cost matrix fails to load.
    pub async fn with_cost_matrix(mut self, source: &CostMatrixSource) -> LoaderResult<Self> {
        match source {
            CostMatrixSource::Predefined(name) => {
                self.cost_model = Some(self.loader.load::<CostModel>(name).await?);
            }
        }
        Ok(self)
    }

    #[must_use]
    pub fn with_cost_model_obj(mut self, model: Arc<CostModel>) -> Self {
        self.cost_model = Some(model);
        self
    }

    /// Adds keycode registry to the session.
    ///
    /// # Errors
    /// Returns `LoaderResult` if the registry fails to load.
    pub async fn with_keycodes(mut self, name: &str) -> LoaderResult<Self> {
        self.registry = Some(self.loader.load::<KeycodeRegistry>(name).await?);
        Ok(self)
    }

    #[must_use]
    pub fn with_rubric(mut self, rubric: Rubric) -> Self {
        self.rubric = Some(Arc::new(rubric));
        self
    }

    #[must_use]
    pub fn with_config(mut self, config: SearchConfig) -> Self {
        self.search_config = Some(config);
        self
    }

    #[must_use]
    pub fn with_biometrics(mut self, samples: Vec<BiometricSample>) -> Self {
        self.biometrics = samples;
        self
    }

    /// Builds the final session.
    ///
    /// # Errors
    /// Returns `LoaderResult` if some required assets are missing or invalid.
    pub fn build(self) -> LoaderResult<ScoringSession> {
        let kb_def = self
            .keyboard
            .ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing keyboard".into()))?;
        let corpus = self
            .corpus
            .ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing corpus".into()))?;
        let rubric = self.rubric.unwrap_or_else(|| Arc::new(Rubric::default()));
        let mut cost_model_val = (*self.cost_model.ok_or_else(|| {
            keyforge_model::error::ForgeError::Config("Missing cost model".into())
        })?)
        .clone();

        let registry = self
            .registry
            .unwrap_or_else(|| Arc::new(KeycodeRegistry::default()));
        let config = self.search_config.unwrap_or_default();

        if !self.biometrics.is_empty() {
            cost_model_val = BiometricProfiler::profile(&self.biometrics, &cost_model_val);
        }
        let cost_model = Arc::new(cost_model_val);

        let keyboard = Arc::new(keyforge_model::Keyboard::new(
            kb_def.geometry.keys.clone(),
            kb_def.geometry.home_row,
            kb_def.meta.kb_type.clone(),
        )?);

        let compilation_ctx = keyforge_physics::EngineCompilationContext {
            keyboard,
            corpus,
            rubric,
            cost_model,
            engine_config: keyforge_model::config::EngineConfig::default(),
        };

        let hw_provider = keyforge_infra::hardware::FsHardwareProvider::default();
        let topo = HardwareProbe::probe_with_provider(Some(&hw_provider));
        let engine = if topo.vendor == "GenuineIntel" {
            keyforge_physics::EngineFactory::new_intel_comet_lake(&compilation_ctx, None)
        } else if topo.vendor == "ARM" {
            keyforge_physics::EngineFactory::new_arm_neon(&compilation_ctx, None)
        } else {
            keyforge_physics::EngineFactory::new_generic(&compilation_ctx)
        }
        .map_err(|e| keyforge_model::error::ForgeError::PhysicsCompute(e.to_string()))?;

        Ok(ScoringSession::new(Arc::from(engine), registry, config))
    }
}
