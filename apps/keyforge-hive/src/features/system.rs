// apps/keyforge-hive/src/features/system.rs

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


use axum::{extract::State, routing::get, Json, Router};
use keyforge_model::Config;
use keyforge_core::loader::AssetLoader;
use keyforge_model::KeyboardDefinition;
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
    let db_status = match sqlx::query("SELECT 1").execute(&state.jobs.repo.pool).await {
        Ok(_) => "connected".to_string(),
        Err(e) => {
            tracing::error!("Health Check DB Fail: {}", e);
            return Err(AppError::ServiceUnavailable("Database Unreachable".into()));
        }
    };

    let queue_depth = state.queue.current_depth().await;
    let assets = if state.assets_healthy.load(std::sync::atomic::Ordering::Relaxed) {
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
    let list = state.assets.list_keyboards().await;
    Ok(Json(list))
}

pub async fn get_keyboard(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> AppResult<Json<KeyboardDefinition>> {
    let kb = state.assets.load_keyboard(&name).await.map_err(|e| {
        tracing::error!("Failed to load keyboard {}: {}", name, e);
        AppError::NotFound
    })?;
    Ok(Json(kb))
}

pub async fn get_app_config(State(state): State<Arc<AppState>>) -> AppResult<Json<Config>> {
    let config: Arc<Config> = state.assets.load_config_asset("config").await;
    Ok(Json(config.as_ref().clone()))
}

pub async fn list_corpora(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let list = state.assets.list_corpora().await;
    Ok(Json(list))
}

pub async fn list_costs(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let list = state.assets.list_cost_matrices().await;
    Ok(Json(list))
}

pub async fn list_keymap_extras(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<String>>> {
    Ok(Json(vec![]))
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
        .route("/api/manifest", get(crate::features::assets::get_manifest))
        .route("/api/data/system/{*path}", get(crate::features::assets::get_asset))
}
