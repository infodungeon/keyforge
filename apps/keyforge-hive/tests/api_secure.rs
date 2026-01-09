// apps/keyforge-hive/tests/api_secure.rs

//! Integration tests for Hive's authenticated REST API endpoints.

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
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;

async fn setup_test_app() -> (axum::Router, Arc<AppState>, sqlx::PgPool, tempfile::TempDir, ContainerAsync<Redis>) {
    // 1. Start Valkey (Redis) Container
    let valkey_node = Redis::default().start().await.expect("Failed to start Valkey");
    let valkey_port = valkey_node.get_host_port_ipv4(6379).await.expect("Failed to get port");
    let valkey_url = format!("redis://127.0.0.1:{}", valkey_port);
    std::env::set_var("KEYFORGE_VALKEY_URL", &valkey_url);

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    std::env::set_var("HIVE_SECRET", "test_secret");

    let pool = init_db(&db_url).await;
    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    let mut config = keyforge_hive::config::AppConfig::mock();
    config.valkey_url = valkey_url;
    config.hive_secret = "test_secret".to_string();

    let state = Arc::new(AppState::new(
        pool.clone(),
        data_path.clone(),
        "test_key".to_string(),
        config.clone()
    ).await);
    let app = create_app(state.clone(), &config, data_path);

    (app, state, pool, temp_dir, valkey_node)
}

fn test_addr() -> SocketAddr {
    "127.0.0.1:1234".parse().unwrap()
}

#[tokio::test]
async fn test_api_user_nuke_unauthorized() {
    let (app, _, _, _temp, _valkey) = setup_test_app().await;

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
    let (app, _, _, _temp, _valkey) = setup_test_app().await;

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
    let (app, state, pool, _temp, _valkey) = setup_test_app().await;

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
