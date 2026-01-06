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

use keyforge_hive::cache::{CompiledEngineCache, GlobalAssetCache};
use keyforge_hive::infra::repositories::{JobRepository, NodeRepository};
use keyforge_hive::VerificationService;
use keyforge_protocol::ResultSubmission;
use keyforge_security as crypto;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[tokio::test]
async fn test_signature_enforcement() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string());
    let pool = keyforge_hive::infra::db::init_db(&db_url).await;
    
    let node_repo = NodeRepository::new(pool.clone());
    let job_repo = JobRepository::new(pool.clone());
    let asset_cache = Arc::new(GlobalAssetCache::new(std::path::PathBuf::from("data")));
    let engine_cache = Arc::new(CompiledEngineCache::new());

    let service = VerificationService::new(job_repo, node_repo.clone(), asset_cache, engine_cache);

    let node_id = format!("node-{}", Uuid::new_v4());
    let (sk_hex, pk_hex) = crypto::generate_keypair();

    // Register node with key
    node_repo.register_heartbeat(&node_id, "CPU", 4, None, 1000.0, Some(&pk_hex)).await.unwrap();

    let job_id = Uuid::new_v4().to_string();
    let layout = "A B C".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Case 1: Bad Signature
    let sub_bad = ResultSubmission {
        version: 1,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score: 100.0,
        node_id: node_id.clone(),
        timestamp,
        nonce: 123,
        signature: Some("bad_sig".into()),
    };
    assert!(service.verify_submission(&sub_bad).await.is_err());

    // Case 2: Valid Signature
    let sig = crypto::sign_result(&sk_hex, &job_id, &layout, 100.0, timestamp, 123).unwrap();
    let sub_good = ResultSubmission {
        version: 1,
        job_id: job_id.clone(),
        layout: layout.clone(),
        score: 100.0,
        node_id: node_id.clone(),
        timestamp,
        nonce: 123,
        signature: Some(sig),
    };
    
    // This should fail at the DB lookup step (job not found), proving signature passed
    let res = service.verify_submission(&sub_good).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Not Found");
}