use crate::error::{AppError, AppResult};
use crate::infra::repositories::audit::AuditLog;
use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Deserialize;
use std::sync::Arc;
use tracing::warn;

use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct NukeRequest {
    pub username: String,
    pub confirmation: String,
}

#[utoipa::path(
    post,
    path = "/user/nuke",
    request_body = NukeRequest,
    responses(
        (status = 200, description = "User data erased"),
        (status = 400, description = "Invalid confirmation")
    ),
    tag = "user"
)]
pub async fn nuke_user_data(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NukeRequest>,
) -> AppResult<String> {
    if payload.confirmation != "DELETE_EVERYTHING" {
        return Err(AppError::Validation("Invalid confirmation string".into()));
    }

    let user_id: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(&payload.username)
            .fetch_optional(&state.users.pool)
            .await
            .map_err(AppError::Database)?;

    if let Some(uid) = user_id {
        // Audit Log
        state
            .audit
            .log(AuditLog {
                action: "USER_NUKE",
                actor_id: Some(uid),
                target: Some("ALL_DATA"),
                details: Some(serde_json::json!({"username": payload.username})),
                ip: None,
                status_code: None,
                request_id: None,
                user_agent: None,
            })
            .await
            .ok();

        // FIX: Parameterized Query
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uid)
            .execute(&state.users.pool)
            .await
            .map_err(AppError::Database)?;

        warn!(
            "☢️ NUKE: User {} ({}) deleted all data.",
            payload.username, uid
        );
        Ok("User data erased successfully.".to_string())
    } else {
        Err(AppError::NotFound)
    }
}
