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

use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct NodeRepository {
    pool: Pool<Postgres>,
}

impl NodeRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Registers a node's long-term identity and hardware profile.
    /// Note: Liveness heartbeats are now handled by the DistributedCoordinator (Valkey).
    /// This method should only be called on startup or major status changes.
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
        if let Some(new_key) = public_key {
            if let Ok(Some(existing_key)) = self.get_public_key(node_id).await {
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
        let row: Option<Option<String>> =
            sqlx::query_scalar("SELECT public_key FROM nodes WHERE id = $1")
                .bind(node_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.flatten())
    }

    /// DEPRECATED: Use DistributedCoordinator::count_active_nodes() instead.
    /// Kept for fallback/historical analysis.
    #[deprecated(note = "Use DistributedCoordinator::count_active_nodes")]
    pub async fn count_recent(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM nodes WHERE last_seen > NOW() - INTERVAL '1 minute'",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    /// DEPRECATED: Use DistributedCoordinator metrics.
    #[deprecated(note = "Use DistributedCoordinator::get_cluster_stats")]
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
}
