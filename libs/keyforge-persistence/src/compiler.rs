use crate::error::PersistenceError;
use keyforge_adapter::loader::AssetLoader;
use keyforge_model::config::Config;
use keyforge_model::constants::paths::ASSET_KEYCODES;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::EngineRequest;
use keyforge_protocol::CostModelDto;
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
        .load::<KeycodeRegistry>(ASSET_KEYCODES)
        .await
        .map_err(|e| PersistenceError::AssetLoad(format!("Keycodes: {e}")))?;

    // 4. Load Cost Model
    let cost_model = match &config.cost_matrix {
        keyforge_model::CostMatrixSource::Predefined(name) => {
            let dto = loader
                .load::<CostModelDto>(name)
                .await
                .map_err(|e| PersistenceError::AssetLoad(format!("Cost Model {name}: {e}")))?;
            Arc::new((*dto).clone().into())
        }
    };

    // 5. Translate to Physics entities using Adapter
    let keyboard =
        keyforge_adapter::conversion::to_domain_keyboard(&kb_def.geometry).map_err(|e| {
            keyforge_model::error::ForgeError::InvalidData(format!("Keyboard geometry: {e}"))
        })?;

    let rubric = keyforge_adapter::conversion::to_domain_rubric(&config.weights);

    let adapter_config =
        keyforge_adapter::conversion::to_domain_config(&config.search, config.seed.unwrap_or(0));

    // Initial layout: Use "default" or fallback to "qwerty" if present
    let initial_layout_str = kb_def
        .layouts
        .get("default")
        .or_else(|| kb_def.layouts.get("qwerty"));

    let initial_layout = if let Some(layout_str) = initial_layout_str {
        Some(
            keyforge_adapter::conversion::parse_layout_string_strict(
                layout_str,
                keyboard.count(),
                &registry,
            )
            .map_err(|e| {
                keyforge_model::error::ForgeError::InvalidData(format!("Initial layout: {e}"))
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
        engine_config: config.engine,
        initial_layout,
        pinned_keys,
    })
}
