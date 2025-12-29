use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct NodeRepository {
    pool: Pool<Postgres>,
}

impl NodeRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn register_heartbeat(
        &self,
        node_id: &str,
        cpu_model: &str,
        cores: i32,
        l2_cache_kb: Option<i32>,
        ops_per_sec: f32,
        public_key: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        // App-Side TOFU (Trust On First Use)
        // If the node exists and has a key, ensure the new key matches.
        if let Some(new_key) = public_key {
            if let Ok(Some(existing_key)) = self.get_public_key(node_id).await {
                // Normalize keys (strip whitespace) for comparison
                let existing_clean = existing_key.trim();
                let new_clean = new_key.trim();

                if !existing_clean.is_empty() && existing_clean != new_clean {
                    tracing::warn!("🚨 Security Alert: Node Identity Mismatch for {}", node_id);
                    return Err(sqlx::Error::Protocol(
                        "Node Identity Mismatch: Public Key changed".into(),
                    ));
                }
            }
        }

        sqlx::query("SELECT register_node_heartbeat($1, $2, $3, $4, $5, $6, $7)")
            .bind(node_id)
            .bind(cpu_model)
            .bind(std::env::consts::ARCH)
            .bind(cores)
            .bind(l2_cache_kb)
            .bind(ops_per_sec)
            .bind(public_key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_public_key(&self, node_id: &str) -> Result<Option<String>, sqlx::Error> {
        // Fix: Explicitly decode as Option<String> to handle NULLs in the DB
        let row: Option<Option<String>> =
            sqlx::query_scalar("SELECT public_key FROM nodes WHERE id = $1")
                .bind(node_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.flatten())
    }

    pub async fn count_recent(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM nodes WHERE last_seen > NOW() - INTERVAL '1 minute'",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    pub async fn sum_ops(&self) -> Result<f32, sqlx::Error> {
        let ops: f32 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(performance_rating), 0) FROM nodes WHERE last_seen > NOW() - INTERVAL '1 minute'",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(ops)
    }

    pub async fn prune_inactive_nodes(&self, minutes: i32) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM nodes WHERE last_seen < NOW() - make_interval(mins => $1)")
                .bind(minutes)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected())
    }

    pub async fn touch_node(&self, node_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE nodes SET last_seen = NOW() WHERE id = $1")
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
