// apps/keyforge-hive/tests/websockets.rs

//! Integration tests for Hive WebSocket communication.

use futures::{SinkExt, StreamExt};
use keyforge_hive::{create_app, infra::db::init_db, state::AppState};
use keyforge_protocol::constants::WS_MSG_JOB;
use std::net::SocketAddr;
use std::sync::Arc;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

// Ensure tracing is initialized only once
use std::sync::Once;
static INIT: Once = Once::new();

fn init_test_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("info,keyforge_hive=debug")
            .with_test_writer()
            .init();
    });
}

async fn start_test_server() -> (String, Arc<AppState>, ContainerAsync<Redis>) {
    init_test_tracing();

    // Start Valkey
    let valkey_node = Redis::default()
        .start()
        .await
        .expect("Failed to start Valkey");
    let valkey_port = valkey_node
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let valkey_url = format!("redis://127.0.0.1:{}", valkey_port);
    std::env::set_var("KEYFORGE_VALKEY_URL", &valkey_url);

    // Force HTTP
    std::env::remove_var("TLS_CERT");
    std::env::remove_var("TLS_KEY");

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });
    let pool = init_db(&db_url).await;
    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    let mut config = keyforge_hive::config::AppConfig::mock();
    config.valkey_url = valkey_url;

    let state =
        Arc::new(AppState::new(pool, data_path.clone(), "test_key".into(), config.clone()).await);
    let app = create_app(state.clone(), &config, data_path);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (
        format!("ws://{}:{}", "127.0.0.1", addr.port()),
        state,
        valkey_node,
    )
}

#[tokio::test]
async fn test_websocket_lifecycle() {
    let (ws_url, state, _valkey) = start_test_server().await;
    let url = Url::parse(&ws_url)
        .unwrap()
        .join("ws?node_id=test-node")
        .unwrap();

    let (ws_stream, _) = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        connect_async(url.to_string()),
    )
    .await
    .expect("Connection timed out")
    .expect("Failed to connect");

    // 1. Connection Verification (Ping/Pong Barrier)
    let (mut sink, mut stream) = ws_stream.split();

    let ping_task = tokio::spawn(async move {
        for _ in 0..50 {
            if sink
                .send(Message::Ping(vec![1, 2, 3].into()))
                .await
                .is_err()
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        sink
    });

    let mut pong_found = false;
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            msg = stream.next() => {
                if let Some(Ok(Message::Pong(p))) = msg {
                    if p == vec![1, 2, 3] { pong_found = true; break; }
                }
            }
            _ = &mut timeout => break,
        }
    }

    let sink = ping_task.await.unwrap();
    let mut ws_stream = stream.reunite(sink).unwrap();
    assert!(pong_found, "Server did not respond to Ping");

    // 2. Broadcast Verification (Signal Spam)
    let job_id = "123";
    let state_clone = state.clone();

    let signal_task = tokio::spawn(async move {
        for _ in 0..50 {
            let _ = state_clone.tx.send(format!("{}{}", WS_MSG_JOB, job_id));
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });

    let mut job_found = false;
    let timeout_job = tokio::time::sleep(std::time::Duration::from_secs(3));
    tokio::pin!(timeout_job);

    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    // FIX: Check for "Job" (Capitalized) or just the ID
                    if text.contains(job_id) && (text.contains("Job") || text.contains("job")) {
                        job_found = true;
                        break;
                    }
                }
            }
            _ = &mut timeout_job => break,
        }
    }

    signal_task.abort();
    assert!(job_found, "Did not receive Job broadcast");
}
