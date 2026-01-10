// apps/keyforge-assets/tests/integration.rs

//! Integration tests for the KeyForge Asset Server.
//! Verifies that assets are correctly served from the distributed store.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use keyforge_assets::create_app;
use keyforge_infra::{DistributedCoordinator, ValkeyProvider};
use std::sync::Arc;
use tower::ServiceExt;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

#[tokio::test]
async fn test_manifest_endpoint() {
    // 1. Start Valkey
    let valkey_node = Redis::default().start().await.expect("Failed to start Valkey");
    let valkey_port = valkey_node.get_host_port_ipv4(6379).await.expect("Failed to get port");
    let valkey_url = format!("redis://127.0.0.1:{}", valkey_port);

    // 2. Setup Coordinator & Provider
    let coordinator = Arc::new(DistributedCoordinator::new(&valkey_url).await.unwrap());
    let provider = Arc::new(ValkeyProvider::new(coordinator.clone()));

    // 3. Create App
    let app = create_app(provider);

    // 4. Request /manifest
    let response = app
        .oneshot(
            Request::builder()
                .uri("/manifest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
