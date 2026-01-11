// apps/keyforge-assets/src/main.rs

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use clap::Parser;
use keyforge_infra::{DistributedCoordinator, ValkeyProvider};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Parser)]
struct Args {
    #[arg(long, env = "PORT", default_value_t = 3001)]
    port: u16,

    #[arg(long, env = "KEYFORGE_VALKEY_URL", default_value = "redis://127.0.0.1:6379")]
    valkey_url: String,
}

struct AppState {
    provider: ValkeyProvider,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("🔌 Connecting to Valkey at {}", args.valkey_url);
    let coordinator = Arc::new(
        DistributedCoordinator::new(&args.valkey_url)
            .await
            .map_err(|e| anyhow::anyhow!("Valkey connection failed: {}", e))?,
    );

    let provider = ValkeyProvider::new(coordinator);
    let state = Arc::new(AppState { provider });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/manifest", get(get_manifest))
        .route("/data/{*path}", get(get_asset))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("🚀 Asset Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn get_manifest(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let manifest = state.provider.get_manifest().await;
    Json(manifest)
}

async fn get_asset(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    // Basic path sanitization is handled by axum Path (decoding), 
    // but we should ensure no weird traversals just in case.
    if path.contains("..") {
        return StatusCode::BAD_REQUEST.into_response();
    }

    match state.provider.get_file_content(&path).await {
        Some(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], bytes).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
