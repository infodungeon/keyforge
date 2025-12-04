use axum::{
    routing::{get, post},
    Json, Router,
};
use keyforge_core::config::ScoringWeights;
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

// Mock Hive State
struct MockHive {
    job_served: Mutex<bool>,
    result_received: Mutex<bool>,
}

#[tokio::test]
async fn test_node_worker_flow() {
    let hive_state = Arc::new(MockHive {
        job_served: Mutex::new(false),
        result_received: Mutex::new(false),
    });

    // Create explicit clones for each closure to consume
    let state_for_queue = hive_state.clone();
    let state_for_results = hive_state.clone();

    // 1. Setup Mock Hive Server
    let app = Router::new()
        .route(
            "/jobs/queue",
            get(move || {
                let state = state_for_queue.clone();
                async move {
                    let mut served = state.job_served.lock().unwrap();
                    if *served {
                        // Only serve once, then return empty (wait)
                        Json(json!({ "job_id": null, "config": null }))
                    } else {
                        *served = true;
                        // Serve a dummy job
                        Json(json!({
                            "job_id": "job-123",
                            "config": {
                                "geometry": create_mock_geometry(),
                                "weights": ScoringWeights::default(),
                                "pinned_keys": "",
                                "corpus_name": "test_corpus"
                            }
                        }))
                    }
                }
            }),
        )
        .route(
            "/jobs/job-123/population",
            get(|| async {
                // Return empty population (random start)
                Json(json!({ "layouts": [] }))
            }),
        )
        .route(
            "/results",
            post(move |Json(_payload): Json<serde_json::Value>| {
                let state = state_for_results.clone();
                async move {
                    let mut received = state.result_received.lock().unwrap();
                    *received = true;
                    "Accepted"
                }
            }),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 0)); // Random port
    let listener = TcpListener::bind(addr).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", port);

    // Run Server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 2. Run Worker (Simulated)
    let client = reqwest::Client::new();

    // Step A: Fetch Queue
    let resp = client
        .get(format!("{}/jobs/queue", server_url))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["job_id"], "job-123");

    // Step B: Submit Result
    let res_resp = client
        .post(format!("{}/results", server_url))
        .json(&json!({
            "job_id": "job-123",
            "layout": "abc...",
            "score": 100.0,
            "node_id": "test-node"
        }))
        .send()
        .await
        .unwrap();

    assert!(res_resp.status().is_success());

    // Verify State
    assert!(*hive_state.job_served.lock().unwrap());
    assert!(*hive_state.result_received.lock().unwrap());
}

fn create_mock_geometry() -> KeyboardGeometry {
    let keys = (0..10)
        .map(|i| KeyNode {
            id: format!("k{}", i),
            hand: 0,
            finger: 0,
            row: 0,
            col: i as i8,
            x: i as f32,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            is_stretch: false,
        })
        .collect();

    KeyboardGeometry {
        keys,
        prime_slots: vec![],
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    }
}
