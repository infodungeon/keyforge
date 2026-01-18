// apps/keyforge-hive/src/api/auth.rs

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

use crate::auth::hash_key;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{extract::State, http::HeaderMap, Json};
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Request payload for registering a new user.
#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    /// Desired username. Must be unique.
    pub username: String,
}

/// Request payload for creating a new API key.
#[derive(Deserialize, ToSchema)]
pub struct CreateKeyRequest {
    /// Human-readable label for the key (e.g., "Laptop", "CI/CD").
    pub label: String,
}

/// Response payload containing a newly generated API key and its hash.
#[derive(Serialize, ToSchema)]
pub struct ApiKeyResponse {
    /// The plaintext API key. This is only returned once upon generation.
    pub api_key: String,
    /// The SHA-256 hash of the API key, for reference.
    pub key_hash: String,
}

/// Helper to generate a secure random key
fn generate_secure_key() -> (String, String) {
    let api_key: String = OsRng
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let key_prefix = "kf_";
    let full_key = format!("{key_prefix}{api_key}");
    let hash = hash_key(&full_key);
    (full_key, hash)
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered", body = ApiKeyResponse),
        (status = 409, description = "Username already taken")
    ),
    tag = "auth"
)]
/// Registers a new user account and returns a master API key.
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<ApiKeyResponse>> {
    // 1. Create User (Fail if exists)
    let user_id = state
        .users
        .create_user(&payload.username)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Conflict("Username already taken".into()))?;

    // 2. Generate Master Key
    let (full_key, hash) = generate_secure_key();

    // 3. Store Hash
    state
        .users
        .create_api_key(user_id, &hash, "Master Key")
        .await
        .map_err(AppError::Database)?;

    Ok(Json(ApiKeyResponse {
        api_key: full_key,
        key_hash: hash,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/keys",
    request_body = CreateKeyRequest,
    responses(
        (status = 200, description = "New key generated", body = ApiKeyResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "auth",
    security(("api_key" = []))
)]
/// Generates a new API key for the authenticated user.
pub async fn generate_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateKeyRequest>,
) -> AppResult<Json<ApiKeyResponse>> {
    // 1. Authenticate User via Header
    let auth_header = headers
        .get("X-Keyforge-Secret")
        .ok_or(AppError::Validation("Missing Auth Header".into()))?
        .to_str()
        .map_err(|_| AppError::Validation("Invalid Header".into()))?;

    let current_hash = hash_key(auth_header);

    let user_id = state
        .users
        .get_user_by_key_hash(&current_hash)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Validation("Invalid API Key".into()))?;

    // 2. Generate New Key
    let (full_key, hash) = generate_secure_key();

    // 3. Store Hash linked to same user
    state
        .users
        .create_api_key(user_id, &hash, &payload.label)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(ApiKeyResponse {
        api_key: full_key,
        key_hash: hash,
    }))
}
