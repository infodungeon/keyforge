use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use keyforge_infra::ServerManifest;
use std::path::{Component, Path as StdPath};
use std::sync::Arc;
use tracing::info;

pub async fn get_manifest(State(state): State<Arc<AppState>>) -> AppResult<Json<ServerManifest>> {
    // Serve from memory cache
    if let Some(manifest) = state.assets.get_manifest() {
        Ok(Json(manifest.as_ref().clone()))
    } else {
        // Fallback (should not happen if warm_all succeeded)
        info!("⚠️ Manifest cache miss, generating from disk...");
        let system_root = state.data_path.join("system");
        let manifest = keyforge_infra::generate_manifest(&system_root)
            .map_err(|e| AppError::Any(anyhow::anyhow!(e)))?;
        Ok(Json(manifest))
    }
}

pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    // SECURITY: Robust Path Traversal Check
    // We parse the path components to ensure no '..' or root references exist.
    let path_obj = StdPath::new(&path);
    if path_obj.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Serve from memory cache
    if let Some(bytes) = state.assets.get_file_content(&path) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();

        Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(bytes))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
