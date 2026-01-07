// apps/keyforge-hive/src/features/assets.rs

use crate::error::AppResult;
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

pub async fn get_manifest(State(state): State<Arc<AppState>>) -> AppResult<Json<ServerManifest>> {
    let manifest = state.assets.get_manifest().await;
    Ok(Json(manifest))
}

pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    let path_obj = StdPath::new(&path);
    if path_obj.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(bytes) = state.assets.get_file_content(&path).await {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();

        Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(bytes))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
