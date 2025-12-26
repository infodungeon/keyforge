use crate::state::AppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use keyforge_infra::AssetLoader;
use keyforge_model::Corpus;
use keyforge_protocol::config::CorpusSource;
use std::sync::Arc;

/// GET /api/corpus/:name
/// Returns a fully-processed Corpus struct for the given corpus name
pub async fn get_corpus(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Corpus>, StatusCode> {
    // Load the corpus using the existing FsProvider logic via GlobalAssetCache
    let corpus = state
        .assets
        .load_corpus(&[CorpusSource {
            id: name.clone(),
            weight: 1.0,
            hash: None,
        }])
        .map_err(|e| {
            tracing::error!("Failed to load corpus '{}': {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(corpus))
}
