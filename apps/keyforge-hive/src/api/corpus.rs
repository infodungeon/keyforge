// apps/keyforge-hive/src/api/corpus.rs

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


use crate::state::AppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use keyforge_infra::AssetLoader;
use keyforge_model::Corpus;

use std::sync::Arc;

/// GET /api/corpus/:name
/// Returns a fully-processed Corpus struct for the given corpus name
/// Retrieves a fully processed corpus definition by name.
pub async fn get_corpus(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Corpus>, StatusCode> {
    // Load the corpus using the existing FsProvider logic via GlobalAssetCache
    let corpus = state
        .assets
        .load_corpus(&[keyforge_model::config::CorpusSource {
            id: name.clone(),
            weight: 1.0,
            hash: None,
        }])
        .await
        .map_err(|e| {
            tracing::error!("Failed to load corpus '{}': {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(corpus))
}
