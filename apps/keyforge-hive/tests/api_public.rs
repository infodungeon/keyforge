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
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceExt;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;

async fn setup_test_app() -> (axum::Router, Arc<AppState>, ContainerAsync<Redis>) {
    // 1. Start Valkey (Redis) Container
    let valkey_node = Redis::default().start().await.expect("Failed to start Valkey");
    let valkey_port = valkey_node.get_host_port_ipv4(6379).await.expect("Failed to get port");
    let valkey_url = format!("redis://127.0.0.1:{}", valkey_port);
    std::env::set_var("KEYFORGE_VALKEY_URL", &valkey_url);

    // 2. Setup Database
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });
    let pool = init_db(&db_url).await;
    
    // 3. Setup Filesystem
    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    // 4. Initialize App State
    // This will now connect to the Valkey container we just started
    let state = Arc::new(AppState::new(pool, data_path.clone(), "test_key".into()).await);
    let app = create_app(state.clone(), data_path);
    
    // Return container guard to keep it alive
    (app, state, valkey_node)
}

fn test_addr() -> SocketAddr {
    "127.0.0.1:1234".parse().unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let (app, _, _valkey) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .extension(ConnectInfo(test_addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_manifest_access() {
    let (app, _, _valkey) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/manifest")
                .extension(ConnectInfo(test_addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_submissions_list() {
    let (app, _, _valkey) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/submissions")
                .extension(ConnectInfo(test_addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
