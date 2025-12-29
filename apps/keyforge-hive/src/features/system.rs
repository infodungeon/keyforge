use axum::{extract::State, routing::get, Json, Router};
use keyforge_protocol::config::Config;
use keyforge_model::loader::AssetLoader;
use keyforge_protocol::geometry::KeyboardDefinition;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub message: String,
    pub db: String,
    pub queue_depth: usize,
    pub assets: String,
}

/// VSA Feature: System & Diagnostics
/// Provides health checks and metadata listings.

pub async fn root() -> &'static str {
    "KeyForge Hive API v0.8"
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "System health status", body = StatusResponse)
    ),
    tag = "system"
)]
pub async fn health(State(state): State<Arc<AppState>>) -> AppResult<Json<StatusResponse>> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.jobs.pool).await {
        Ok(_) => "connected".to_string(),
        Err(e) => {
            tracing::error!("Health Check DB Fail: {}", e);
            return Err(AppError::ServiceUnavailable("Database Unreachable".into()));
        }
    };

    let queue_depth = state.queue.current_depth().await;

    let assets = if state
        .assets_healthy
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        "ok".to_string()
    } else {
        "degraded".to_string()
    };

    Ok(Json(StatusResponse {
        status: "ok".to_string(),
        version: "0.8.0".to_string(),
        message: "Genetic Reservoir Active".to_string(),
        db: db_status,
        queue_depth,
        assets,
    }))
}

pub async fn list_keyboards(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let list = state.assets.list_keyboards();
    Ok(Json(list))
}

pub async fn get_keyboard(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> AppResult<Json<KeyboardDefinition>> {
    let kb = state.assets.load_keyboard(&name).map_err(|e| {
        tracing::error!("Failed to load keyboard {}: {}", name, e);
        AppError::NotFound
    })?;
    Ok(Json(kb))
}

pub async fn get_app_config(State(state): State<Arc<AppState>>) -> AppResult<Json<Config>> {
    let config = state.assets.load_app_config();
    Ok(Json(config.as_ref().clone()))
}

pub async fn list_corpora(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let list = state.assets.list_corpora();
    Ok(Json(list))
}

pub async fn list_costs(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let list = state.assets.list_cost_matrices();
    Ok(Json(list))
}

pub async fn list_keymap_extras(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<String>>> {
    let list = keyforge_infra::listing::list_keymap_extras(&state.data_path)
        .map_err(|e| AppError::Any(anyhow::anyhow!(e)))?;
    Ok(Json(list))
}

pub fn system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/ws", get(crate::api::ws::handler))
        .route("/sys/metrics", get(crate::api::metrics::get_metrics))
        .route("/sys/status", get(crate::api::metrics::get_system_status))
        .route("/api/keyboards", get(list_keyboards))
        .route("/api/keyboards/{name}", get(get_keyboard))
        .route("/api/corpora", get(list_corpora))
        .route("/api/costs", get(list_costs))
        .route("/api/keymap_extras", get(list_keymap_extras))
}
