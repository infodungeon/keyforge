use crate::error::PersistenceError;
use keyforge_compute::loader::AssetLoader;
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
) -> Result<EngineRequest, PersistenceError> {
    // 1. Load Keyboard Definition
    let kb_def = loader
        .load::<KeyboardDefinition>(&config.keyboard)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keyboard {}: {}", config.keyboard, e)))?;

    // 2. Load Corpus
    let corpus = loader
        .load_corpus(&config.corpora)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Corpus: {e}")))?;

    // 3. Load Keycode Registry (Mandatory for resolution)
    let registry = loader
        .load::<KeycodeRegistry>("keycodes")
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keycodes: {e}")))?;

    // 4. Load Cost Model
    let cost_model = match &config.cost_matrix {
        keyforge_model::CostMatrixSource::Predefined(name) => loader
            .load::<keyforge_model::cost_model::CostModel>(name)
            .await
            .map_err(|e| PersistenceError::AssetLoad(format!("Cost Model {name}: {e}")))?,
    };

    // 5. Translate to Physics entities using Adapter
    let keyboard = keyforge_adapter::conversion::to_domain_keyboard(&kb_def.geometry)
        .map_err(|e| keyforge_model::error::ForgeError::InvalidData(format!("Keyboard geometry: {e}")))?;

    let rubric = keyforge_adapter::conversion::to_domain_rubric(&config.weights);

    let adapter_config =
        keyforge_adapter::conversion::to_domain_config(&config.search, config.seed.unwrap_or(0));

    // Initial layout: Use first defined layout or default to QWERTY if present
    let initial_layout = if let Some(layout_str) = kb_def.layouts.get("qwerty") {
        Some(
            keyforge_adapter::conversion::parse_layout_string_strict(
                layout_str,
                keyboard.count(),
                &registry,
            )
            .map_err(|e| {
                keyforge_model::error::ForgeError::InvalidData(format!(
                    "Default layout 'qwerty': {e}"
                ))
            })?,
        )
    } else {
        None
    };

    let pinned_keys = keyforge_adapter::conversion::resolve_constraints(
        &config.pinned_keys,
        keyboard.count(),
        &registry,
    )
    .map_err(|e| keyforge_model::error::ForgeError::InvalidData(format!("Pinned keys: {e}")))?;

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
