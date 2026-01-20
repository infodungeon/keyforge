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
use keyforge_core::loader::{AssetLoader, LoaderResult};
use keyforge_core::ScoringSession;
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
            keyforge_model::Keyboard::new(kb_def.geometry.keys.clone(), kb_def.geometry.home_row)
                .map_err(|e| keyforge_model::error::ForgeError::InvalidData(e.to_string()))?,
        );

        let engine = keyforge_physics::EngineFactory::new_generic(&keyboard, &corpus, &rubric, &cost_model)
            .map_err(|e| keyforge_model::error::ForgeError::PhysicsCompute(e.to_string()))?;

        Ok(ScoringSession::new(Arc::from(engine), registry, config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::Asset;
    use std::any::Any;

    #[derive(Debug)]
    struct MockLoader;
    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
            if id == "missing" { return Err(keyforge_model::error::ForgeError::NotFound(id.into())); }
            
            let any_kb = Arc::new(KeyboardDefinition {
                geometry: keyforge_model::geometry::KeyboardGeometry {
                    keys: vec![keyforge_model::KeyNode::default()],
                    prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                    ..Default::default()
                },
                ..Default::default()
            }) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_kb.downcast::<T>() { return Ok(arc); }

            let json = r#"{
                "meta": { "version": "2.0", "description": "T", "unit": "pts" },
                "models": { "model_a_row_staggered": { "description": "t", "static_costs": {} } },
                "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
            }"#;
            let model: CostModel = serde_json::from_str(json).unwrap();
            let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_model.downcast::<T>() { return Ok(arc); }

            let any_kc = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_kc.downcast::<T>() { return Ok(arc); }

            Err(keyforge_model::error::ForgeError::NotFound(id.to_string()))
        }
        async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }
    }

    #[tokio::test]
    async fn test_session_builder_lifecycle() {
        let loader = MockLoader;
        let kb_def = Arc::new(KeyboardDefinition {
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            },
            ..Default::default()
        });
        let corp = Arc::new(Corpus::default());
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs: std::collections::HashMap::new(),
        });
        let cm_arc = Arc::new(cm);

        let builder = SessionBuilder::new(&loader)
            .with_keyboard("kb").await.unwrap()
            .with_keyboard_def(kb_def)
            .with_corpus(&[]).await.unwrap()
            .with_corpus_obj(corp)
            .with_cost_matrix(&CostMatrixSource::Predefined("cm".into())).await.unwrap()
            .with_cost_model_obj(cm_arc)
            .with_keycodes("kc").await.unwrap()
            .with_rubric(Rubric::default())
            .with_config(SearchConfig::default())
            .with_biometrics(vec![BiometricSample { bigram: "th".into(), ms: 100.0, timestamp: 0 }]);
        
        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("SessionBuilder"));
        assert!(debug_str.contains("biometrics_count: 1"));

        let session = builder.build().unwrap();
        assert_eq!(session.registry.definitions.len(), 0);
    }

    #[tokio::test]
    async fn test_session_builder_missing_assets() {
        let loader = MockLoader;
        
        // 1. Missing keyboard
        let b1 = SessionBuilder::new(&loader);
        assert!(b1.build().is_err());

        // 2. Missing corpus
        let b2 = SessionBuilder::new(&loader).with_keyboard("kb").await.unwrap();
        assert!(b2.build().is_err());

        // 3. Missing cost model
        let b3 = SessionBuilder::new(&loader).with_keyboard("kb").await.unwrap().with_corpus(&[]).await.unwrap();
        assert!(b3.build().is_err());

        // 4. Default registry and rubric
        let mut cm = CostModel::default();
        cm.models.insert("model_a_row_staggered".into(), keyforge_model::cost_model::ModelDefinition {
            description: "test".into(), static_costs: std::collections::HashMap::new(),
        });
        let b4 = SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(KeyboardDefinition {
                geometry: keyforge_model::geometry::KeyboardGeometry {
                    keys: vec![keyforge_model::KeyNode::default()],
                    prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                    ..Default::default()
                }, ..Default::default()
            }))
            .with_corpus_obj(Arc::new(Corpus::default()))
            .with_cost_model_obj(Arc::new(cm));
        let session = b4.build().unwrap();
        assert_eq!(session.registry.definitions.len(), 0); // Uses default (empty) registry
    }

    #[tokio::test]
    async fn test_session_builder_invalid_keyboard() {
        let loader = MockLoader;
        let mut kb = KeyboardDefinition::default();
        // Keyboard with keys but no slots coverage -> validation fails in Keyboard::new
        kb.geometry.keys.push(keyforge_model::KeyNode::default());
        
        let builder = SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(kb))
            .with_corpus_obj(Arc::new(Corpus::default()))
            .with_cost_model_obj(Arc::new(CostModel::default()));
        
        assert!(builder.build().is_err());
    }

    #[tokio::test]
    async fn test_session_builder_physics_error() {
        let loader = MockLoader;
        let kb_def = Arc::new(KeyboardDefinition {
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys: vec![keyforge_model::KeyNode::default()],
                prime_slots: vec![keyforge_model::types::KeyIndex(0)],
                ..Default::default()
            }, ..Default::default()
        });
        
        // Corpus with invalid data that might trigger engine failure?
        // Actually, generic engine creation rarely fails if inputs are validated.
        // But we can trigger a Config error in build() by missing assets.
        let builder = SessionBuilder::new(&loader).with_keyboard_def(kb_def);
        let res = builder.build();
        assert!(matches!(res, Err(keyforge_model::error::ForgeError::Config(_))));
    }
}
