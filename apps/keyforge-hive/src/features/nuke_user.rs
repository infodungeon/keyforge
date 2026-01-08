// apps/keyforge-hive/src/features/nuke_user.rs

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

use axum::{extract::State, Json};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use tracing::warn;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::infra::repositories::audit::AuditLog;

/// Request payload for permanently deleting all user data.
#[derive(Deserialize, ToSchema)]
pub struct NukeRequest {
    /// The username of the account to be deleted.
    pub username: String,
    /// Confirmation string. Must be exactly "DELETE_EVERYTHING".
    pub confirmation: String,
}

/// VSA Feature: Nuke User Data
/// Permanently erases all data for a specific user.

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
/// Handles a request to permanently delete all data associated with a user account.
pub async fn handle(
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

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uid)
            .execute(&state.users.pool)
            .await
            .map_err(AppError::Database)?;

        warn!(
            "☢️ (VSA) NUKE: User {} ({}) deleted all data.",
            payload.username, uid
        );
        Ok("User data erased successfully.".to_string())
    } else {
        Err(AppError::NotFound)
    }
}
