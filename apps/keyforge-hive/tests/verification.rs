// apps/keyforge-hive/tests/verification.rs

//! Integration tests for submission signature verification.

use keyforge_hive::cache::{CompiledEngineCache, ParsedLayoutCache};
use keyforge_hive::infra::repositories::{JobRepository, NodeRepository};
use keyforge_hive::VerificationService;
use keyforge_infra::{DistributedCoordinator, ValkeyDistributedCoordinator, ValkeyProvider};
use keyforge_protocol::{ResultSubmission, PROTOCOL_VERSION};
use keyforge_security as crypto;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use uuid::Uuid;

#[tokio::test]
async fn test_signature_enforcement() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });
    let pool = keyforge_hive::infra::db::init_db(&db_url).await;

    let node_repo = NodeRepository::new(pool.clone());
    let job_repo = JobRepository::new(pool.clone());

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

    let coordinator: Arc<dyn DistributedCoordinator> = Arc::new(ValkeyDistributedCoordinator::new(&valkey_url).await.unwrap());
    let assets = Arc::new(ValkeyProvider::new(coordinator));

    let engine_cache = Arc::new(CompiledEngineCache::new());
    let layout_cache = Arc::new(ParsedLayoutCache::new());

    let service = VerificationService::new(job_repo, node_repo.clone(), assets, engine_cache, layout_cache);

    let node_id = format!("node-{}", Uuid::new_v4());
    let (sk_hex, pk_hex) = crypto::generate_keypair();

    // Register node with key
    node_repo
        .register_heartbeat(&node_id, "CPU", 4, None, 1000.0, Some(&pk_hex))
        .await
        .unwrap();

    let job_id = Uuid::new_v4().to_string();
    let layout = "A B C".to_string();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Case 1: Bad Signature
    let sub_bad = ResultSubmission {
        version: PROTOCOL_VERSION,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score: 100.0,
        node_id: node_id.clone(),
        timestamp,
        nonce: 123,
        signature: "bad_sig".into(),
    };
    assert!(service.verify_submission(&sub_bad).await.is_err());

    // Case 2: Valid Signature
    let sig = crypto::sign_result(&sk_hex, &job_id, &layout, 100.0, timestamp, 456).unwrap();
    let sub_good = ResultSubmission {
        version: PROTOCOL_VERSION,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score: 100.0,
        node_id: node_id.clone(),
        timestamp,
        nonce: 456,
        signature: sig,
    };

    // This should fail at the DB lookup step (job not found), proving signature passed
    let res = service.verify_submission(&sub_good).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Not Found");
}
