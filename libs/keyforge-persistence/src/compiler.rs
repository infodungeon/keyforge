use crate::error::PersistenceError;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::Config;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_physics::EngineRequest;
use std::sync::Arc;

/// Compiles a high-level `JobRequest` into a high-performance `EngineRequest`.
///
/// # Errors
/// Returns `PersistenceError` if any assets fail to load or if the domain translation fails.
pub async fn compile_request<L: AssetLoader>(
    loader: &L,
    config: &Config,
    keyboard_id: &str,
    corpora_ids: &[&str],
) -> Result<EngineRequest, PersistenceError> {
    // 1. Load Keyboard Definition
    let kb_def = loader
        .load::<KeyboardDefinition>(keyboard_id)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keyboard {keyboard_id}: {e}")))?;

    // 2. Load Corpus
    let corpus_sources: Vec<_> = corpora_ids
        .iter()
        .map(|&id| keyforge_model::config::CorpusSource {
            id: id.to_string(),
            weight: 1.0,
            hash: None,
        })
        .collect();
    let corpus = loader
        .load_corpus(&corpus_sources)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Corpus: {e}")))?;

    // 3. Load Keycode Registry (Mandatory for resolution)
    let registry = loader
        .load::<KeycodeRegistry>("keycodes")
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keycodes: {e}")))?;

    // 4. Load Cost Model
    let cost_model = loader
        .load::<keyforge_model::cost_model::CostModel>("cost_matrix")
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Cost Model: {e}")))?;

    // 5. Translate to Physics entities using Adapter
    let keyboard = keyforge_adapter::conversion::to_domain_keyboard(&kb_def.geometry)
        .map_err(|e| PersistenceError::Adapter(e.to_string()))?;

    let rubric = keyforge_adapter::conversion::to_domain_rubric(&config.weights);

    let adapter_config = keyforge_adapter::conversion::to_domain_config(&config.search, 0);

    // Initial layout: Use first defined layout or default to QWERTY if present
    let initial_layout = if let Some(layout_str) = kb_def.layouts.get("qwerty") {
        Some(
            keyforge_adapter::conversion::parse_layout_string_strict(
                layout_str,
                keyboard.count(),
                &registry,
            )
            .map_err(|e| PersistenceError::Adapter(e.to_string()))?,
        )
    } else {
        None
    };

    let pinned_keys = keyforge_adapter::conversion::resolve_constraints(
        &config.pinned_keys,
        keyboard.count(),
        &registry,
    )
    .map_err(|e| PersistenceError::Adapter(e.to_string()))?;

    Ok(EngineRequest {
        keyboard: Arc::new(keyboard),
        corpus,
        rubric: Arc::new(rubric),
        cost_model,
        config: adapter_config,
        initial_layout,
        pinned_keys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_core::loader::LoaderResult;
    use keyforge_model::cost_model::CostModel;
    use keyforge_model::geometry::KeyboardGeometry;
    use keyforge_model::keycodes::KeycodeRegistry;
    use keyforge_model::{config::CorpusSource, Asset, Corpus};
    use std::any::Any;

    #[derive(Debug)]
    struct MockLoader;

    #[async_trait::async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            let any_kb = Arc::new(KeyboardDefinition {
                meta: Default::default(),
                geometry: KeyboardGeometry {
                    keys: vec![Default::default()],
                    prime_slots: vec![],
                    med_slots: vec![],
                    low_slots: vec![],
                    home_row: 0,
                },
                layouts: Default::default(),
            }) as Arc<dyn Any + Send + Sync>;

            if let Ok(arc) = any_kb.downcast::<T>() {
                return Ok(arc);
            }

            let json = r#"{
                "meta": { "version": "2.0", "description": "Test", "unit": "pts" },
                "models": {
                    "model_a_row_staggered": {
                        "description": "test",
                        "static_costs": {}
                    }
                },
                "dynamic_rules": { "sequence_modifiers": {}, "penalties": {}, "constraints": {} }
            }"#;
            let model: CostModel = serde_json::from_str(json).unwrap();
            let any_model = Arc::new(model) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_model.downcast::<T>() {
                return Ok(arc);
            }

            let any_kc = Arc::new(KeycodeRegistry::default()) as Arc<dyn Any + Send + Sync>;
            if let Ok(arc) = any_kc.downcast::<T>() {
                return Ok(arc);
            }

            Err(keyforge_model::error::ForgeError::NotFound(_id.to_string()))
        }

        async fn load_corpus(&self, _sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }
    }

    #[tokio::test]
    async fn test_compile_request_success() {
        let loader = MockLoader;
        let config = keyforge_model::config::Config::default();
        let res = compile_request(&loader, &config, "test_kb", &[]).await;
        assert!(res.is_ok());
    }
}
