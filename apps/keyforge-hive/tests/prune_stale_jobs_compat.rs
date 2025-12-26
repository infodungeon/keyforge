use keyforge_hive::infra::{db::init_db, repositories::JobRepository};
use uuid::Uuid;

async fn seed_min_job(pool: &sqlx::PgPool, job_id: &str) {
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

    let sc_id: i32 = sqlx::query_scalar(
        "INSERT INTO search_configs (search_epochs, search_steps, search_patience, search_patience_threshold, temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash) VALUES (1,1,1,0.1,0.1,0.1,1,1, $1) RETURNING id",
    )
    .bind(&hash)
    .fetch_one(pool)
    .await
    .expect("Failed to seed SearchConfig");

    sqlx::query("INSERT INTO jobs (id, keyboard_id, scoring_profile_id, search_config_id, pinned_keys, corpus_name, cost_matrix, status, started_at, retry_count) VALUES ($1, $2, $3, $4, '[]', 'default', 'cost.json', 'processing', NOW() - INTERVAL '20 minutes', 0)")
        .bind(job_id)
        .bind(kb_id)
        .bind(sp_id)
        .bind(sc_id)
        .execute(pool)
        .await
        .expect("Failed to seed Job");
}

#[tokio::test]
async fn prune_stale_jobs_tolerates_missing_node_id_column() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    std::env::set_var("DATABASE_MAX_CONNECTIONS", "5");
    let pool = init_db(&db_url).await;

    // Simulate an older / partially migrated schema missing `jobs.node_id`.
    sqlx::query("DROP INDEX IF EXISTS idx_jobs_node_id")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE jobs DROP COLUMN IF EXISTS node_id")
        .execute(&pool)
        .await
        .unwrap();

    let job_id = Uuid::new_v4().to_string();
    seed_min_job(&pool, &job_id).await;

    let repo = JobRepository::new(pool.clone());

    let reset = repo.prune_stale_jobs(10, 3).await.unwrap();
    assert_eq!(reset, 1);

    let status: String = sqlx::query_scalar("SELECT status FROM jobs WHERE id = $1")
        .bind(&job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "active");

    let started_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT started_at FROM jobs WHERE id = $1")
            .bind(&job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(started_at.is_none());
}
