// Copyright (c) 2025 KeyForge Contributors
//
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

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use keyforge_hive::{create_app, infra::db::init_db, state::AppState};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceExt; 

async fn setup_test_app() -> (axum::Router, Arc<AppState>, sqlx::PgPool, tempfile::TempDir) {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    std::env::set_var("HIVE_SECRET", "test_secret");

    let pool = init_db(&db_url).await;
    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    let state = Arc::new(AppState::new(
        pool.clone(),
        data_path.clone(),
        "test_key".to_string(),
    ));
    let app = create_app(state.clone(), data_path);

    (app, state, pool, temp_dir)
}

fn test_addr() -> SocketAddr {
    "127.0.0.1:1234".parse().unwrap()
}

#[tokio::test]
async fn test_api_user_nuke_unauthorized() {
    let (app, _, _, _temp) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/nuke")
                .header("Content-Type", "application/json")
                .extension(ConnectInfo(test_addr()))
                .body(Body::from(json!({"username": "foo", "confirmation": "bar"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_user_nuke_invalid_confirmation() {
    let (app, _, _, _temp) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/nuke")
                .header("Content-Type", "application/json")
                .header("X-Keyforge-Secret", "test_secret")
                .extension(ConnectInfo(test_addr()))
                .body(Body::from(json!({"username": "foo", "confirmation": "WRONG"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_user_nuke_success() {
    let (app, state, pool, _temp) = setup_test_app().await;

    let username = format!("nuke_target_{}", uuid::Uuid::new_v4());
    state.users.create_user(&username).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/nuke")
                .header("Content-Type", "application/json")
                .header("X-Keyforge-Secret", "test_secret")
                .extension(ConnectInfo(test_addr()))
                .body(Body::from(json!({"username": username, "confirmation": "DELETE_EVERYTHING"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(exists.is_none());
}