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

use keyforge_hive::infra::{db::init_db, repositories::JobRepository, queue::WriteQueue, repositories::ResultRepository};
use keyforge_hive::cache::GlobalAssetCache;
use uuid::Uuid;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

async fn seed_min_job(pool: &sqlx::PgPool, job_id: &str) {
    let hash = Uuid::new_v4().to_string();

    let kb_id: i32 = sqlx::query_scalar("INSERT INTO keyboards (name, author, version, unique_hash) VALUES ('dummy', 'tester', $1, $2) RETURNING id")
        .bind(&hash).bind(&hash).fetch_one(pool).await.expect("Failed to seed Keyboard");

    let sp_id: i32 = sqlx::query_scalar("INSERT INTO scoring_profiles (weights, config_hash) VALUES ('{}'::jsonb, $1) RETURNING id")
        .bind(&hash).fetch_one(pool).await.expect("Failed to seed Profile");

    let sc_id: i32 = sqlx::query_scalar("INSERT INTO search_configs (search_epochs, search_steps, search_patience, search_patience_threshold, temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash) VALUES (1,1,1,0.1,0.1,0.1,1,1, $1) RETURNING id")
        .bind(&hash).fetch_one(pool).await.expect("Failed to seed SearchConfig");

    sqlx::query("INSERT INTO jobs (id, keyboard_id, scoring_profile_id, search_config_id, pinned_keys, corpus_name, cost_matrix, status, started_at, retry_count) VALUES ($1, $2, $3, $4, '[]', 'default', 'cost.json', 'processing', NOW() - INTERVAL '70 minutes', 0)")
        .bind(job_id).bind(kb_id).bind(sp_id).bind(sc_id)
        .execute(pool).await.expect("Failed to seed Job");
}

#[tokio::test]
async fn prune_stale_jobs_resets_zombies() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string());
    let pool = init_db(&db_url).await;
    let job_id = Uuid::new_v4().to_string();
    seed_min_job(&pool, &job_id).await;

    let repo = JobRepository::new(pool.clone());
    let reset = repo.prune_stale_jobs(10, 3).await.unwrap();
    
    // Legacy schema fallback test (if node_id column is missing, it resets based on time)
    // Note: If schema is fully migrated, this tests normal behavior.
    assert!(reset >= 1);

    let status: String = sqlx::query_scalar("SELECT status FROM jobs WHERE id = $1")
        .bind(&job_id).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "active");
}

#[derive(Serialize, Deserialize, Clone)]
struct PersistedRecord {
    job_id: String,
    layout: String,
    score: f32,
    node_id: String,
}

#[derive(Serialize, Deserialize)]
struct WalEntry {
    checksum: u32,
    record: PersistedRecord,
}

#[tokio::test]
async fn test_wal_recovery_integration() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string());
    let pool = init_db(&db_url).await;
    let repo = ResultRepository::new(pool.clone());

    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();
    let queue_dir = data_path.join("user/queue");
    std::fs::create_dir_all(&queue_dir).unwrap();
    std::fs::create_dir_all(data_path.join("system/config")).unwrap();

    let job_id = Uuid::new_v4().to_string();
    let node_id = Uuid::new_v4().to_string();
    
    // Seed Job dependencies for foreign key constraints
    seed_min_job(&pool, &job_id).await;
    sqlx::query("INSERT INTO nodes (id, cpu_cores, performance_rating) VALUES ($1, 1, 1.0) ON CONFLICT (id) DO NOTHING")
        .bind(&node_id).execute(&pool).await.unwrap();

    // Create WAL file
    let record = PersistedRecord { job_id: job_id.clone(), layout: "A B C".into(), score: 123.45, node_id: node_id.clone() };
    let record_bytes = postcard::to_stdvec(&record).unwrap();
    let checksum = crc32fast::hash(&record_bytes);
    let entry = WalEntry { checksum, record };
    let bytes = postcard::to_stdvec(&entry).unwrap();
    std::fs::write(queue_dir.join(format!("{}.bin", Uuid::new_v4())), bytes).unwrap();

    // Start Queue (Should recover WAL)
    let assets = Arc::new(GlobalAssetCache::new(data_path.clone()));
    let _queue = WriteQueue::new(repo.clone(), data_path.clone(), assets);

    sleep(Duration::from_secs(2)).await;

    let best = repo.get_best_score(&job_id).await.unwrap();
    assert_eq!(best, Some(123.45));
}