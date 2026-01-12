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

    let provider = Arc::new(ValkeyProvider::new(coordinator));
    
    // Create app using the generic provider
    let app = keyforge_assets::create_app(provider);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("🚀 Asset Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    "OK"
}
