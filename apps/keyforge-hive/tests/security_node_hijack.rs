use keyforge_hive::infra::repositories::NodeRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_node_id_hijack_prevention() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://keyforge:forge_password@localhost:5432/keyforge_hive".to_string()
    });

    std::env::set_var("DATABASE_MAX_CONNECTIONS", "5");
    let pool = keyforge_hive::infra::db::init_db(&db_url).await;

    // Apply Security Patch (TOFU)
    sqlx::query(r#"
        CREATE OR REPLACE FUNCTION register_node_heartbeat(
            p_node_id TEXT,
            p_cpu_model TEXT,
            p_arch TEXT,
            p_cores INTEGER,
            p_l2_cache INTEGER,
            p_ops_per_sec REAL,
            p_public_key TEXT
        ) RETURNS VOID AS $$ BEGIN
        INSERT INTO hardware_profiles (
            cpu_signature, architecture, l2_cache_kb, verified_ops_per_sec, updated_at
        ) VALUES (
            p_cpu_model, p_arch, p_l2_cache, p_ops_per_sec, CURRENT_TIMESTAMP
        ) ON CONFLICT (cpu_signature) DO UPDATE
        SET verified_ops_per_sec = GREATEST(hardware_profiles.verified_ops_per_sec, EXCLUDED.verified_ops_per_sec),
            l2_cache_kb = COALESCE(EXCLUDED.l2_cache_kb, hardware_profiles.l2_cache_kb),
            updated_at = CURRENT_TIMESTAMP;

        INSERT INTO nodes (
            id, cpu_signature, cpu_cores, performance_rating, last_seen, public_key
        ) VALUES (
            p_node_id, p_cpu_model, p_cores, p_ops_per_sec, CURRENT_TIMESTAMP, p_public_key
        ) ON CONFLICT (id) DO UPDATE
        SET last_seen = CURRENT_TIMESTAMP,
            performance_rating = EXCLUDED.performance_rating,
            cpu_cores = EXCLUDED.cpu_cores,
            public_key = COALESCE(nodes.public_key, EXCLUDED.public_key);
        END;
        $$ LANGUAGE plpgsql;
    "#).execute(&pool).await.expect("Failed to apply security patch");

    let repo = NodeRepository::new(pool.clone());
    let node_id = format!("test-node-{}", Uuid::new_v4());
    let key_a = "KEY_A_PAYLOAD";
    let key_b = "KEY_B_PAYLOAD_ROGUE";

    // 1. Initial Registration (Success)
    repo.register_heartbeat(&node_id, "TestCPU", 4, None, 1000.0, Some(key_a))
        .await
        .unwrap();

    // 2. Same Node, Same Key (Success - Heartbeat)
    repo.register_heartbeat(&node_id, "TestCPU", 4, None, 1000.0, Some(key_a))
        .await
        .unwrap();

    // 3. Same Node, Different Key (Hijack Attempt - SHOULD FAIL)
    let res = repo
        .register_heartbeat(&node_id, "TestCPU", 4, None, 1000.0, Some(key_b))
        .await;

    // ASSERTION: This must fail for the test to pass
    assert!(
        res.is_err(),
        "Hijack attempt should have failed but succeeded!"
    );

    // 4. Verify Integrity
    let stored_key = repo.get_public_key(&node_id).await.unwrap();
    assert_eq!(
        stored_key.as_deref(),
        Some(key_a),
        "SECURITY FAILURE: Public key was overwritten by rogue update!"
    );
}
