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
use keyforge_model::{KeyboardDefinition, CostModel, KeycodeRegistry};
use keyforge_physics::EngineRequest;
use std::sync::Arc;
use keyforge_adapter::conversion::{to_domain_rubric, to_domain_config, resolve_constraints};

/// Compiles a raw configuration into a fully-loaded `EngineRequest`.
///
/// # Errors
///
/// Returns `PersistenceError` if any assets (keyboard, corpus, cost model) fail to load or validate.
pub async fn compile_request<L: AssetLoader>(
    loader: &L,
    config: &Config,
    keyboard_name: &str,
    pinned_keys: &[keyforge_model::KeyConstraint],
) -> Result<EngineRequest, PersistenceError> {
    // 1. Load Keyboard
    let keyboard_def = loader
        .load::<KeyboardDefinition>(keyboard_name)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keyboard '{keyboard_name}': {e}")))?;

    let keyboard = keyforge_model::Keyboard::new(
        keyboard_def.geometry.keys.clone(),
        keyboard_def.geometry.home_row,
    )
    .map_err(|e| PersistenceError::Validation(e.to_string()))?;

    // 2. Load Corpus
    // Use default if no corpora specified (Config currently doesn't hold corpora list)
    let corpus_sources: Vec<CorpusSource> = vec![CorpusSource::default()]; 
    let corpus = loader
        .load_corpus(&corpus_sources)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Corpus: {e}")))?;

    // 3. Load Cost Model
    let CostMatrixSource::Predefined(cost_name) = CostMatrixSource::default();
    
    let cost_model = loader
        .load::<CostModel>(&cost_name)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("CostModel '{cost_name}': {e}")))?;

    // 4. Load Keycodes (Standard)
    let registry = loader
        .load::<KeycodeRegistry>("keycodes")
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keycodes: {e}")))?;

    // 5. Construct Request
    let pinned = resolve_constraints(pinned_keys, keyboard.count(), &registry)
        .map_err(|e| PersistenceError::Validation(e.to_string()))?;

    let seed = config.search.seed.unwrap_or(0);

    Ok(EngineRequest {
        keyboard: Arc::new(keyboard),
        corpus,
        rubric: Arc::new(to_domain_rubric(&config.weights)),
        cost_model,
        config: to_domain_config(&config.search, seed),
        initial_layout: None,
        pinned_keys: pinned,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::error::ForgeError;
    use keyforge_core::loader::LoaderResult;
    use keyforge_model::{Asset, Corpus};

    #[derive(Debug)]
    struct FailingLoader;

    #[async_trait::async_trait]
    impl AssetLoader for FailingLoader {
        async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>> {
            Err(ForgeError::NotFound(id.to_string()))
        }
        async fn load_corpus(&self, _: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Err(ForgeError::NotFound("corpus".into()))
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
