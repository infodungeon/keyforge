use keyforge_hive::cache::GlobalAssetCache;
use keyforge_hive::infra::queue::WriteQueue;
use keyforge_hive::infra::repositories::ResultRepository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tokio::time::{sleep, Duration};
use uuid::Uuid;

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

async fn seed_dependencies(pool: &sqlx::PgPool, job_id: &str, node_id: &str) {
    sqlx::query("INSERT INTO nodes (id, cpu_cores, performance_rating) VALUES ($1, 1, 1.0) ON CONFLICT (id) DO NOTHING")
        .bind(node_id)
        .execute(pool).await.expect("Failed to seed Node");

    let hash = Uuid::new_v4().to_string();

    let kb_id: i32 = sqlx::query_scalar(
        "INSERT INTO keyboards (name, author, version, unique_hash) VALUES ('dummy', 'tester', $1, $2) RETURNING id",
    )
    .bind(&hash)
    .bind(&hash)
    .fetch_one(pool)
    .await
    .expect("Failed to seed Keyboard");

    let sp_id: i32 = sqlx::query_scalar(
        "INSERT INTO scoring_profiles (weights, config_hash) VALUES ('{}'::jsonb, $1) RETURNING id",
    )
    .bind(&hash)
    .fetch_one(pool)
    .await
    .expect("Failed to seed Profile");

    let sc_id: i32 = sqlx::query_scalar("INSERT INTO search_configs (search_epochs, search_steps, search_patience, search_patience_threshold, temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash) VALUES (1,1,1,0.1,0.1,0.1,1,1, $1) RETURNING id")
        .bind(&hash)
        .fetch_one(pool).await.expect("Failed to seed SearchConfig");

    sqlx::query("INSERT INTO jobs (id, keyboard_id, scoring_profile_id, search_config_id, pinned_keys, corpus_name, cost_matrix) VALUES ($1, $2, $3, $4, '[]', 'default', 'cost.json')")
        .bind(job_id)
        .bind(kb_id)
        .bind(sp_id)
        .bind(sc_id)
        .execute(pool).await.expect("Failed to seed Job");
}

#[tokio::test]
async fn test_wal_recovery() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    std::env::set_var("DATABASE_MAX_CONNECTIONS", "5");
    let pool = keyforge_hive::infra::db::init_db(&db_url).await;

    let repo = ResultRepository::new(pool.clone());

    let temp_dir = tempfile::tempdir().unwrap();
    let data_path = temp_dir.path().to_path_buf();

    // Create user/queue directory
    let queue_dir = data_path.join("user/queue");
    std::fs::create_dir_all(&queue_dir).unwrap();

    // Also need system/config for hive.json defaults if loaded
    std::fs::create_dir_all(data_path.join("system/config")).unwrap();

    let job_id = Uuid::new_v4().to_string();
    let node_id = Uuid::new_v4().to_string();
    let wal_id = Uuid::new_v4();

    seed_dependencies(&pool, &job_id, &node_id).await;

    let record = PersistedRecord {
        job_id: job_id.clone(),
        layout: "A B C".into(),
        score: 123.45,
        node_id: node_id.clone(),
    };

    let record_bytes = bincode::serialize(&record).unwrap();
    let checksum = crc32fast::hash(&record_bytes);

    let entry = WalEntry { checksum, record };

    let bytes = bincode::serialize(&entry).unwrap();
    std::fs::write(queue_dir.join(format!("{}.bin", wal_id)), bytes).unwrap();

    let assets = Arc::new(GlobalAssetCache::new(data_path.clone()));
    let _queue = WriteQueue::new(repo.clone(), data_path.clone(), assets);

    sleep(Duration::from_secs(2)).await;

    let best = repo.get_best_score(&job_id).await.unwrap();
    assert_eq!(
        best,
        Some(123.45),
        "Record should be recovered and inserted into DB"
    );
}
