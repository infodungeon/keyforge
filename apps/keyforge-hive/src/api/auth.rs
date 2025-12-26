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

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateKeyRequest {
    pub label: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub api_key: String,
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
    let full_key = format!("{}{}", key_prefix, api_key);
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
        .ok_or_else(|| AppError::Validation("Username already taken".into()))?;

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
