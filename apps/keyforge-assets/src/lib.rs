// apps/keyforge-assets/src/lib.rs

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use keyforge_infra::asset::AssetServerProvider;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Debug)]
pub struct AppState {
    pub provider: Arc<dyn AssetServerProvider + Send + Sync>,
}

pub fn create_app(provider: Arc<dyn AssetServerProvider + Send + Sync>) -> Router {
    let state = Arc::new(AppState { provider });

    Router::new()
        .route("/health", get(health_check))
        .route("/manifest", get(get_manifest))
        .route("/data/{*path}", get(get_asset))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn get_manifest(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.provider.get_manifest().await {
        Ok(m) => Json(m).into_response(),
        Err(e) => {
            tracing::error!("Manifest error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load manifest").into_response()
        }
    }
}

async fn get_asset(Path(path): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    if path.contains("..") {
        return StatusCode::BAD_REQUEST.into_response();
    }

    match state.provider.get_file_content(&path).await {
        Ok(Some(bytes)) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], bytes).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Asset error for {}: {}", path, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
