// apps/keyforge-hive/tests/integration_valkey.rs

//! Integration tests for Hive Valkey (Redis) telemetry storage.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use futures::SinkExt;
use keyforge_hive::{create_app, infra::db::init_db, state::AppState};
use keyforge_protocol::NodeTelemetry;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

// Ensure tracing is initialized only once
use std::sync::Once;
static INIT: Once = Once::new();

fn init_test_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("info,keyforge_hive=debug,keyforge_infra=debug")
            .with_test_writer()
            .init();
    });
}

#[tokio::test(flavor = "multi_thread")]
async fn test_valkey_telemetry_flow() {
    init_test_tracing();

    // 1. Start Valkey Container
    let valkey_node = Redis::default()
        .start()
        .await
        .expect("Failed to start Valkey");
    let valkey_port = valkey_node
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let valkey_url = format!("redis://127.0.0.1:{valkey_port}");
    println!("Valkey running at {valkey_url}");

    // 2. Setup Hive App
    std::env::set_var("KEYFORGE_VALKEY_URL", &valkey_url);
    // Mock DB (not used for this test but required for startup)
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });
    let pool = init_db(&db_url).await;
    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    let mut config = keyforge_hive::config::AppConfig::mock();
    config.valkey_url = valkey_url;

        let state = Arc::new(AppState::new(pool, data_path.clone(), "test_key".into(), config.clone()).await.expect("Failed to init state"));

    
    let app = create_app(state.clone(), &config, data_path);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hive_url = format!("ws://127.0.0.1:{}", addr.port());
    println!("Hive listening at {hive_url}");

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // 3. Connect WebSocket Client
    let node_id = "test-node-valkey";
    let ws_url = Url::parse(&hive_url)
        .unwrap()
        .join(&format!("ws?node_id={node_id}"))
        .unwrap();

    let (mut ws_stream, _) = connect_async(ws_url.to_string())
        .await
        .expect("Failed to connect");

    // 4. Send Telemetry
    let telemetry = NodeTelemetry {
        job_id: Some("job-123".into()),
        ips: 5000.0,
        temp: 10.5,
        current_best: Some(100.0),
        memory_usage: 1024,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let json = serde_json::to_string(&telemetry).unwrap();
    ws_stream
        .send(Message::Text(json.into()))
        .await
        .expect("Failed to send telemetry");

    // 5. Verify Persistence in Coordinator
    // Give it a moment to process
    println!("Waiting for telemetry processing...");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let stored = state
        .coordinator
        .get_heartbeat(node_id)
        .await
        .expect("Failed to get heartbeat");

    if stored.is_none() {
        println!("Heartbeat missing. Checking logs...");
    }

    assert!(stored.is_some(), "Heartbeat not found in Valkey");

    let t = stored.unwrap();
    assert_eq!(t.job_id, Some("job-123".into()));
    assert_eq!(t.ips, 5000.0);

    // 6. Verify Active Node Count
    let count = state
        .coordinator
        .count_active_nodes()
        .await
        .expect("Failed to count nodes");
    assert_eq!(count, 1);
}
