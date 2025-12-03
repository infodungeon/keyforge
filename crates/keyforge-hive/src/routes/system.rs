use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub message: String,
}

pub async fn root() -> &'static str {
    "KeyForge Hive API v0.7"
}

pub async fn health() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".to_string(),
        version: "0.7.0".to_string(),
        message: "Genetic Reservoir Active".to_string(),
    })
}