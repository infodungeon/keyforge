#[keyforge_testing_macros::kf_test]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::float_cmp,
    clippy::similar_names,
    clippy::print_stdout
)]
mod integration_tests {
    use super::*;

    #[keyforge_testing_macros::kf_test]
    // apps/keyforge-hive/tests/integration_cron.rs

    // Integration tests for Hive's background cron jobs and WAL recovery.
    use keyforge_hive::infra::{
        db::init_db, queue::WriteQueue, repositories::JobRepository, repositories::ResultRepository,
    };
    use keyforge_infra::asset::ValkeyProvider;
    use keyforge_infra::net::distributed::{DistributedCoordinator, ValkeyDistributedCoordinator};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use testcontainers_modules::redis::Redis;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use tokio::time::{sleep, Duration};
    use uuid::Uuid;

    async fn seed_min_job(pool: &sqlx::PgPool, job_id: &str) {
        let hash = Uuid::new_v4().to_string();

        let kb_id: i32 = sqlx::query_scalar!(
            "INSERT INTO keyboards (name, author, version, unique_hash) VALUES ('dummy', 'tester', $1, $2) RETURNING id",
            hash,
            hash
        )
        .fetch_one(pool)
        .await
        .expect("Failed to seed Keyboard");

        let profile_id: i32 = sqlx::query_scalar!(
            "INSERT INTO scoring_profiles (weights, config_hash) VALUES ('{}'::jsonb, $1) RETURNING id",
            hash
        )
        .fetch_one(pool)
        .await
        .expect("Failed to seed Profile");

        let config_id: i32 = sqlx::query_scalar!(
            "INSERT INTO search_configs (search_epochs, search_steps, search_patience, search_patience_threshold, temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash) VALUES (1,1,1,0.1,0.1,0.1,1,1, $1) RETURNING id",
            hash
        )
        .fetch_one(pool)
        .await
        .expect("Failed to seed SearchConfig");

        sqlx::query!(
            "INSERT INTO jobs (id, keyboard_id, scoring_profile_id, search_config_id, pinned_keys, corpus_name, cost_matrix, status, started_at, retry_count) VALUES ($1, $2, $3, $4, '[]', 'default', 'cost.json', 'processing', NOW() - INTERVAL '70 minutes', 0)",
            job_id,
            kb_id,
            profile_id,
            config_id
        )
        .execute(pool)
        .await
        .expect("Failed to seed Job");
    }

    #[tokio::test]
    async fn prune_stale_jobs_resets_zombies() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
        });
        let pool = init_db(&db_url).await;
        let job_id = Uuid::new_v4().to_string();
        seed_min_job(&pool, &job_id).await;

        let repo = JobRepository::new(pool.clone());
        let reset = repo.prune_stale_jobs(10, 3).await.unwrap();

        // Legacy schema fallback test (if node_id column is missing, it resets based on time)
        // Note: If schema is fully migrated, this tests normal behavior.
        assert!(reset >= 1);

        let status: String = sqlx::query_scalar!("SELECT status FROM jobs WHERE id = $1", job_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .expect("Job status missing");
        assert_eq!(status, "active");
    }

    #[derive(Serialize, Deserialize, Clone)]
    struct PersistedRecord {
        job_id: String,
        layout: String,
        score: f32,
        raw_score: i64,
        node_id: String,
    }

    #[derive(Serialize, Deserialize)]
    struct WalEntry {
        checksum: u32,
        record: PersistedRecord,
    }

    #[tokio::test]
    async fn test_wal_recovery_integration() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
        });
        let pool = init_db(&db_url).await;
        let repo = ResultRepository::new(pool.clone(), 50);

        let temp_dir = tempfile::tempdir().unwrap();
        let data_path = temp_dir.path().to_path_buf();
        let queue_dir = data_path.join("user/queue");
        std::fs::create_dir_all(&queue_dir).unwrap();
        std::fs::create_dir_all(data_path.join("system/config")).unwrap();

        let job_id = Uuid::new_v4().to_string();
        let node_id = Uuid::new_v4().to_string();

        // Seed Job dependencies for foreign key constraints
        seed_min_job(&pool, &job_id).await;
        sqlx::query!("INSERT INTO nodes (id, cpu_cores, performance_rating) VALUES ($1, 1, 1.0) ON CONFLICT (id) DO NOTHING", node_id)
        .execute(&pool).await.unwrap();

        // Create WAL file
        let record = PersistedRecord {
            job_id: job_id.clone(),
            layout: "A B C".into(),
            score: 123.45,
            raw_score: 123_450_000,
            node_id: node_id.clone(),
        };
        let record_bytes = postcard::to_stdvec(&record).unwrap();
        let checksum = crc32fast::hash(&record_bytes);
        let entry = WalEntry { checksum, record };
        let bytes = postcard::to_stdvec(&entry).unwrap();
        std::fs::write(queue_dir.join(format!("{}.bin", Uuid::new_v4())), bytes).unwrap();

        // Start Valkey
        let valkey_node = Redis::default()
            .start()
            .await
            .expect("Failed to start Valkey");
        let valkey_port = valkey_node
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get port");
        let valkey_url = format!("redis://127.0.0.1:{valkey_port}");

        let coordinator: Arc<dyn DistributedCoordinator> = Arc::new(
            ValkeyDistributedCoordinator::new(&valkey_url)
                .await
                .unwrap(),
        );
        let _assets = Arc::new(ValkeyProvider::new(coordinator));

        // Start Queue (Should recover WAL)
        use keyforge_hive::config::QueueConfig;
        let _queue = WriteQueue::new(repo.clone(), data_path.clone(), QueueConfig::default());

        sleep(Duration::from_secs(2)).await;

        let best = repo.get_best_score(&job_id).await.unwrap();
        assert_eq!(best, Some(123.45));
    }
}
