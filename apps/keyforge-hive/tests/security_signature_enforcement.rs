use keyforge_hive::cache::{CompiledEngineCache, GlobalAssetCache};
use keyforge_hive::infra::repositories::JobRepository;
use keyforge_hive::infra::repositories::NodeRepository;
use keyforge_hive::services::verification::VerificationService;
use keyforge_protocol::ResultSubmission;

use keyforge_security as crypto;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[tokio::test]
async fn test_signature_enforcement() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    std::env::set_var("DATABASE_MAX_CONNECTIONS", "5");
    let pool = keyforge_hive::infra::db::init_db(&db_url).await;

    // Clean state
    sqlx::query("TRUNCATE nodes, jobs, results CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    let node_repo = NodeRepository::new(pool.clone());
    let job_repo = JobRepository::new(pool.clone());
    let asset_cache = Arc::new(GlobalAssetCache::new(std::path::PathBuf::from("data")));
    let engine_cache = Arc::new(CompiledEngineCache::new());

    let service = VerificationService::new(
        job_repo.clone(),
        node_repo.clone(),
        asset_cache,
        engine_cache,
    );

    let node_id_no_key = format!("node-no-key-{}", Uuid::new_v4());
    let node_id_with_key = format!("node-with-key-{}", Uuid::new_v4());

    // Generate Keypair
    let (sk_hex, pk_hex) = crypto::generate_keypair();
    let pk_str = pk_hex.clone();

    // 1. Register Node WITHOUT Key
    node_repo
        .register_heartbeat(&node_id_no_key, "TestCPU", 4, None, 1000.0, None)
        .await
        .unwrap();

    // 2. Register Node WITH Key
    node_repo
        .register_heartbeat(&node_id_with_key, "TestCPU", 4, None, 1000.0, Some(&pk_str))
        .await
        .unwrap();

    let job_id = Uuid::new_v4().to_string(); // Dummy ID
    let layout = "A B C".to_string();
    let score = 100.0;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let nonce = 12345;

    // SCENARIO 1: Node has no key -> Should Fail "Public Key Required"
    let sub_no_key = ResultSubmission {
        version: 1,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score,
        node_id: node_id_no_key.clone(),
        timestamp,
        nonce,
        signature: Some("dummy_sig".into()),
    };

    let res = service.verify_submission(&sub_no_key).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Validation error: Unregistered Node Identity: Public Key Required"
    );

    // SCENARIO 2: Node has key, but submission missing signature -> Should Fail "Missing Signature"
    let sub_missing_sig = ResultSubmission {
        version: 1,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score,
        node_id: node_id_with_key.clone(),
        timestamp,
        nonce,
        signature: None,
    };

    let res = service.verify_submission(&sub_missing_sig).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Validation error: Missing Signature"
    );

    // SCENARIO 3: Node has key, invalid signature -> Should Fail "Crypto Error" or "Invalid Signature"
    let sub_bad_sig = ResultSubmission {
        version: 1,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score,
        node_id: node_id_with_key.clone(),
        timestamp,
        nonce,
        signature: Some("invalid_base64_sig".to_string()), // Malformed
    };

    let res = service.verify_submission(&sub_bad_sig).await;
    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Crypto Error") || err_str.contains("Invalid Signature"));

    // SCENARIO 4: Valid Signature
    let signature =
        crypto::sign_result(&sk_hex, &job_id, &layout, score, timestamp, nonce).unwrap();
    let sub_valid = ResultSubmission {
        version: 1,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score,
        node_id: node_id_with_key.clone(),
        timestamp,
        nonce,
        signature: Some(signature),
    };

    let res = service.verify_submission(&sub_valid).await;
    assert!(res.is_err());
    // It should fail at step 2 (Database/Job extraction), which proves step 1 (Crypto) passed.
    let err_msg = res.unwrap_err().to_string();
    assert_eq!(err_msg, "Not Found");
}
