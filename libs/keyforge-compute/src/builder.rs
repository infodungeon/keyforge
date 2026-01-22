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
use crate::loader::{AssetLoader, LoaderResult};
use crate::session::ScoringSession;
use keyforge_model::config::{CorpusSource, CostMatrixSource};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
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
    ///
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
    ///
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
    ///
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
    ///
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
    ///
    /// Returns `LoaderResult` if some required assets are missing or invalid.
    pub fn build(self) -> LoaderResult<ScoringSession> {
        let kb_def = self
            .keyboard
            .ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing keyboard".into()))?;
        let corpus = self
            .corpus
            .ok_or_else(|| keyforge_model::error::ForgeError::Config("Missing corpus".into()))?;
        let rubric = self.rubric.unwrap_or_else(|| Arc::new(Rubric::default()));
        let mut cost_model = (*self.cost_model.ok_or_else(|| {
            keyforge_model::error::ForgeError::Config("Missing cost model".into())
        })?)
        .clone();
        let registry = self
            .registry
            .unwrap_or_else(|| Arc::new(KeycodeRegistry::default()));
        let config = self.search_config.unwrap_or_default();

        // Task-ui-024: Apply biometric profiling if available
        if !self.biometrics.is_empty() {
            cost_model = BiometricProfiler::profile(&self.biometrics, &cost_model);
        }

        // Create Keyboard from Definition (using home_row from geometry)
        let keyboard = Arc::new(
            keyforge_model::Keyboard::new(
                kb_def.geometry.keys.clone(),
                kb_def.geometry.home_row,
                kb_def.meta.kb_type.clone(),
            )
            .map_err(|e| keyforge_model::error::ForgeError::InvalidData(e.to_string()))?,
        );

        let compilation_ctx = keyforge_physics::EngineCompilationContext {
            keyboard: &keyboard,
            corpus: &corpus,
            rubric: &rubric,
            cost_model: &cost_model,
        };

        let topo = HardwareProbe::probe();
        let engine = if topo.vendor == "GenuineIntel" {
            keyforge_physics::EngineFactory::new_intel_comet_lake(
                compilation_ctx,
                Some(topo.into()),
            )
        } else if topo.vendor == "ARM" {
            keyforge_physics::EngineFactory::new_arm_neon(
                compilation_ctx,
                Some(topo.into()),
            )
        } else {
            keyforge_physics::EngineFactory::new_generic(compilation_ctx)
        }
        .map_err(|e| keyforge_model::error::ForgeError::PhysicsCompute(e.to_string()))?;

        // Task-phys-rev-015: Validate corpus coverage against registry
        let total_freq: u64 = corpus.char_freqs.iter().sum();
        if total_freq > 0 {
            for (i, &freq) in corpus.char_freqs.iter().enumerate() {
                if freq > 0 {
                    let Ok(code) = u16::try_from(i) else { continue };
                    if registry
                        .get_label(keyforge_model::KeyCode(code))
                        .contains("0x")
                    {
                        // Not found in registry (falls back to 0xHEX)
                        // Precision loss in logging is acceptable
                        #[allow(clippy::cast_precision_loss)]
                        let pct = (freq as f64 / total_freq as f64) * 100.0;
                        if pct > 0.1 {
                            tracing::warn!(
                                "Corpus character code {} (weighted {:.2}%) is not in the keycode registry. It will be ignored during optimization!",
                                code, pct
                             );
                        }
                    }
                }
            }
        }

        Ok(ScoringSession::new(Arc::from(engine), registry, config))
    }
}
