use crate::error::PersistenceError;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::Config;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::EngineRequest;
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
