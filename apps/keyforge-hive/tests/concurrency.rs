// apps/keyforge-hive/tests/concurrency.rs

//! # Concurrency Tests for KeyForge Hive
//!
//! Stress testing suite for the Hive server under high concurrent loads.


use futures::future::join_all;
use keyforge_model::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_model::CostMatrixSource;
use keyforge_protocol::{JobConfig, JobRequest, JobResponse, NodeRequest, ResultSubmission, PROTOCOL_VERSION};
use reqwest::{header, Client};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod common;
use common::*;

const NODE_COUNT: usize = 50;
const RESULTS_PER_NODE: usize = 100;

/// Performs a "thundering herd" stress test with multiple worker nodes
/// registering and submitting results simultaneously to ensure system stability.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_heterogeneous_thundering_herd() {
    let (base_url, _state, _temp_dir, _valkey) = setup_server().await;
    let data_root = _temp_dir.path();

    let mut headers = header::HeaderMap::new();
    let mut val = header::HeaderValue::from_static(TEST_SECRET);
    val.set_sensitive(true);
    headers.insert("X-Keyforge-Secret", val);

    let client = Client::builder()
        .default_headers(headers)
        .pool_max_idle_per_host(NODE_COUNT)
        .build()
        .unwrap();

    println!(" Starting Real-World Stress Test");
    println!("   Target: {}", base_url);

    let kb_corne = load_keyboard(data_root, "corne");
    let kb_szr = load_keyboard(data_root, "szr35");

    let weights_std = ScoringWeights::default();
    let mut weights_alt = ScoringWeights::default();
    weights_alt.weights.insert("penalty_scissor".to_string(), 500.0);

    let jobs_config = vec![
        (kb_corne.clone(), weights_std.clone(), "Corne-Std"),
        (kb_corne.clone(), weights_alt.clone(), "Corne-Alt"),
        (kb_szr.clone(), weights_std.clone(), "SZR-Std"),
        (kb_szr.clone(), weights_alt.clone(), "SZR-Alt"),
    ];

    let mut job_ids = Vec::new();

    for (kb, w, label) in jobs_config {
        let req = JobRequest {
            version: PROTOCOL_VERSION,
            config: JobConfig {
                definition: kb,
                weights: w,
                params: SearchParams::default(),
                pinned_keys: vec![],
                corpora: vec![CorpusSource {
                    id: "default".to_string(),
                    weight: 1.0,
                    hash: None,
                }],
                cost_matrix: CostMatrixSource::Predefined("cost_matrix.json".into()),
                biometrics: vec![],
                parent_job_id: None,
                baseline_score: None,
                parents: vec![],
            },
        };

        let resp = client
            .post(format!("{}/jobs", base_url))
            .json(&req)
            .send()
            .await
            .expect("Failed to send request");

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            panic!("❌ Job Registration Failed: {} - {}", status, text);
        }

        let body: JobResponse = resp.json().await.expect("Failed to parse JobResponse");
        println!("   📝 Registered {}: {}", label, &body.job_id[0..8]);
        job_ids.push(body.job_id);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for i in 0..NODE_COUNT {
        let client_ref = client.clone();
        let url_ref = base_url.clone();
        let job_idx = i % 4;
        let target_job = job_ids[job_idx].clone();

        handles.push(tokio::spawn(async move {
            let node_id = format!("worker-{}-job-{}", i, job_idx);

            let reg_req = NodeRequest {
                version: PROTOCOL_VERSION,
                node_id: node_id.clone(),
                cpu_model: "RustTestCPU".into(),
                cores: 8,
                l2_cache_kb: Some(1024),
                ops_per_sec: 1_000_000.0,
                public_key: None,
            };

            // Retry loop for registration
            let mut attempts = 0;
            loop {
                let reg_resp = client_ref
                    .post(format!("{}/nodes/register", url_ref))
                    .json(&reg_req)
                    .send()
                    .await
                    .expect("Reg failed");

                if reg_resp.status().is_success() {
                    break;
                }

                attempts += 1;
                if attempts >= 5 {
                    return Err(format!("Node {} reg failed after 5 attempts: {}", i, reg_resp.status()));
                }
                tokio::time::sleep(std::time::Duration::from_millis(100 * attempts)).await;
            }

            for k in 0..RESULTS_PER_NODE {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let res_req = ResultSubmission {
                    version: PROTOCOL_VERSION,
                    job_id: target_job.clone(),
                    layout: format!("Q W E R T Y U I O P A S D F G H J K L ; {} {}", i, k),
                    score: 500.0,
                    node_id: node_id.clone(),
                    timestamp,
                    nonce: fastrand::u64(..),
                    signature: "dummy".to_string(),
                };

                let res_resp = client_ref
                    .post(format!("{}/results", url_ref))
                    .json(&res_req)
                    .send()
                    .await
                    .expect("Submit failed");

                let status = res_resp.status();

                if status.is_server_error() {
                    let txt = res_resp.text().await.unwrap_or_default();
                    return Err(format!("Node {} crash: {} - {}", i, status, txt));
                }
            }
            Ok(())
        }));
    }

    let results = join_all(handles).await;
    let duration = start_time.elapsed();

    let mut failures = 0;
    for res in results {
        if let Ok(Err(e)) = res {
            println!("   ⚠️  Worker Error: {}", e);
            failures += 1;
        }
    }

    assert_eq!(failures, 0, "Clients encountered errors");

    let total_reqs = NODE_COUNT * RESULTS_PER_NODE;
    println!(
        "✅ Processed {} requests across 4 distinct jobs in {:.2}s",
        total_reqs,
        duration.as_secs_f32()
    );
}