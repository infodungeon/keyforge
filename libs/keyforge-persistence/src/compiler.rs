// libs/keyforge-persistence/src/compiler.rs

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

use crate::error::PersistenceError;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::{Config, CorpusSource, CostMatrixSource};
use keyforge_physics::EngineRequest;
use std::sync::Arc;

/// Compiles a raw configuration into a fully-loaded `EngineRequest`.
pub async fn compile_request(
    loader: &dyn AssetLoader,
    _config: &Config,
    keyboard_name: &str,
    _pinned_keys: &[keyforge_model::KeyConstraint],
) -> Result<EngineRequest, PersistenceError> {
    // 1. Load Keyboard
    let keyboard_def = loader
        .load_keyboard(keyboard_name)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keyboard '{}': {}", keyboard_name, e)))?;

    let keyboard = keyforge_model::Keyboard::new(
        keyboard_def.geometry.keys.clone(),
        keyboard_def.geometry.home_row,
    )
    .map_err(|e| PersistenceError::Validation(e.to_string()))?;

    // 2. Load Corpus
    let corpus_sources: Vec<CorpusSource> = vec![CorpusSource::default()]; // TODO: From config
    let corpus = loader
        .load_corpus(&corpus_sources)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Corpus: {}", e)))?;

    // 3. Load Cost Model
    let cost_name = match CostMatrixSource::default() {
        CostMatrixSource::Predefined(name) => name,
    };
    
    let cost_model = loader
        .load_cost_model(&cost_name)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("CostModel '{}': {}", cost_name, e)))?;

    // 4. Load Keycodes (Standard)
    let _registry = loader
        .load_keycodes("keycodes")
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keycodes: {}", e)))?;

    // 5. Construct Request
    let pinned = vec![None; keyboard.count()];

    Ok(EngineRequest {
        keyboard: Arc::new(keyboard),
        corpus,
        rubric: Arc::new(keyforge_model::Rubric::default()), // TODO: From config
        cost_model,
        config: keyforge_model::SearchConfig::default(), // TODO: From config
        initial_layout: None,
        pinned_keys: pinned,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::error::ForgeError;
    use keyforge_core::loader::LoaderResult;
    use keyforge_model::geometry::KeyboardDefinition;
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::cost_model::CostModel;
    use keyforge_model::Corpus;

    #[derive(Debug)]
    struct FailingLoader;

    #[async_trait::async_trait]
    impl AssetLoader for FailingLoader {
        async fn load_keyboard(&self, _: &str) -> LoaderResult<Arc<KeyboardDefinition>> {
            Err(ForgeError::NotFound("kb".into()))
        }
        async fn load_corpus(&self, _: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Err(ForgeError::NotFound("corpus".into()))
        }
        async fn load_cost_model(&self, _: &str) -> LoaderResult<Arc<CostModel>> {
            Err(ForgeError::NotFound("costs".into()))
        }
        async fn load_keycodes(&self, _: &str) -> LoaderResult<Arc<KeycodeRegistry>> {
            Err(ForgeError::NotFound("keycodes".into()))
        }
    }

    #[tokio::test]
    async fn test_compile_failure_propagation() {
        let loader = FailingLoader;
        let config = Config::default();
        let res = compile_request(&loader, &config, "test_kb", &[]).await;
        assert!(matches!(res, Err(PersistenceError::AssetLoad(_))));
    }
}
