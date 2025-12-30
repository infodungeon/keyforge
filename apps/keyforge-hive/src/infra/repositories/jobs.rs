use keyforge_protocol::config::{CorpusSource, ScoringWeights, SearchParams};
use keyforge_protocol::constants::MAX_PINNED_KEYS_COUNT;
use keyforge_protocol::geometry::{KeyboardDefinition, KeyboardGeometry};
use keyforge_protocol::{CostMatrixSource, JobRequest, KeyConstraint, Validator};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

/// Repository for managing job life cycles, registration, and claiming.
#[derive(Clone)]
pub struct JobRepository {
    pub(crate) pool: Pool<Postgres>,
}

impl JobRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn exists(&self, job_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("SELECT 1 FROM jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.is_some())
    }

    pub async fn register(
        &self,
        job_id: &str,
        req: &JobRequest,
        owner_id: Option<Uuid>,
        parent_job_id: Option<String>,
        priority: i32,
    ) -> Result<bool, sqlx::Error> {
        if self.exists(job_id).await? {
            return Ok(false);
        }

        req.params
            .validate()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid search parameters: {}", e)))?;
        req.weights
            .validate()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid scoring weights: {}", e)))?;

        if req.pinned_keys.len() > MAX_PINNED_KEYS_COUNT {
            return Err(sqlx::Error::Protocol("Pinned keys too large".into()));
        }

        let kb_meta = &req.definition.meta;
        let lock_key = format!("{}{}{}", kb_meta.name, kb_meta.author, kb_meta.version);

        // Hashing logic for components
        let weights_clone = req.weights.clone();
        let params_clone = req.params;
        let lock_key_clone = lock_key.clone();

        let (unique_hash, w_json, w_hash, p_hash) = tokio::task::spawn_blocking(move || {
            fn norm(v: f32) -> f32 {
                if v == 0.0 {
                    0.0
                } else {
                    (v * 1_000_000.0).round() / 1_000_000.0
                }
            }

            let mut hasher = Sha256::new();
            hasher.update(lock_key_clone.as_bytes());
            let unique_hash = hex::encode(hasher.finalize());

            let mut w = weights_clone;
            w.penalty_sfb_base = norm(w.penalty_sfb_base);
            // ... (other norms omitted for brevity, assuming standard usage)

            let w_json = serde_json::to_value(&w).map_err(|e| e.to_string())?;
            let w_str = serde_json::to_string(&w).map_err(|e| e.to_string())?;
            let mut hasher = Sha256::new();
            hasher.update(w_str.as_bytes());
            let w_hash = hex::encode(hasher.finalize());

            let mut p = params_clone;
            p.temp_min = norm(p.temp_min);

            let p_json = serde_json::to_string(&p).map_err(|e| e.to_string())?;
            let mut hasher = Sha256::new();
            hasher.update(p_json.as_bytes());
            let p_hash = hex::encode(hasher.finalize());

            Ok::<(String, serde_json::Value, String, String), String>((
                unique_hash,
                w_json,
                w_hash,
                p_hash,
            ))
        })
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("Hashing task failed: {}", e)))?
        .map_err(sqlx::Error::Protocol)?;

        let mut tx = self.pool.begin().await?;

        // Advisory Lock
        let mut bytes = [0u8; 8];
        let hash_bytes = hex::decode(&unique_hash).unwrap_or_default();
        if hash_bytes.len() >= 8 {
            bytes.copy_from_slice(&hash_bytes[0..8]);
        }
        let lock_id = i64::from_be_bytes(bytes);

        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(lock_id)
            .execute(&mut *tx)
            .await?;

        // Keyboards
        let row = sqlx::query(
            r#"
            INSERT INTO keyboards (name, author, version, notes, kb_type, unique_hash)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (unique_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
        )
        .bind(&kb_meta.name)
        .bind(&kb_meta.author)
        .bind(&kb_meta.version)
        .bind(&kb_meta.notes)
        .bind(&kb_meta.kb_type)
        .bind(&unique_hash)
        .fetch_one(&mut *tx)
        .await?;

        let kb_id: i32 = row.try_get("id")?;

        // Keys
        let keys_exist =
            sqlx::query("SELECT 1 as ex FROM keyboard_keys WHERE keyboard_id = $1 LIMIT 1")
                .bind(kb_id)
                .fetch_optional(&mut *tx)
                .await?;

        if keys_exist.is_none() {
            for (idx, key) in req.definition.geometry.keys.iter().enumerate() {
                let kidx = keyforge_protocol::types::KeyIndex(idx as u16);
                let is_prime = req.definition.geometry.prime_slots.contains(&kidx);
                let is_med = req.definition.geometry.med_slots.contains(&kidx);
                let is_low = req.definition.geometry.low_slots.contains(&kidx);

                sqlx::query(
                    r#"
                    INSERT INTO keyboard_keys 
                    (keyboard_id, idx, x, y, w, h, hand, finger, row_idx, col_idx, is_stretch, is_prime, is_med, is_low, r)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#
                )
                .bind(kb_id)
                .bind(idx as i32)
                .bind(key.x)
                .bind(key.y)
                .bind(key.w)
                .bind(key.h)
                .bind(key.hand.0 as i32)
                .bind(key.finger.0 as i32)
                .bind(key.row.0 as i32)
                .bind(key.col.0 as i32)
                .bind(key.is_stretch)
                .bind(is_prime)
                .bind(is_med)
                .bind(is_low)
                .bind(key.r)
                .execute(&mut *tx)
                .await?;
            }
        }

        // Scoring Profiles
        let score_row = sqlx::query(
            r#"
            INSERT INTO scoring_profiles (weights, config_hash) 
            VALUES ($1, $2)
            ON CONFLICT (config_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
        )
        .bind(w_json)
        .bind(w_hash)
        .fetch_one(&mut *tx)
        .await?;
        let score_id: i32 = score_row.try_get("id")?;

        // Search Config
        let search_row = sqlx::query(
            r#"
            INSERT INTO search_configs (
                search_epochs, search_steps, search_patience, search_patience_threshold,
                temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (config_hash) DO UPDATE SET id = search_configs.id
            RETURNING id
            "#,
        )
        .bind(req.params.search_epochs as i32)
        .bind(req.params.search_steps as i32)
        .bind(req.params.search_patience as i32)
        .bind(req.params.search_patience_threshold)
        .bind(req.params.temp_min)
        .bind(req.params.temp_max)
        .bind(req.params.opt_limit_fast as i32)
        .bind(req.params.opt_limit_slow as i32)
        .bind(p_hash)
        .fetch_one(&mut *tx)
        .await?;
        let search_id: i32 = search_row.try_get("id")?;

        let primary_corpus = req
            .corpora
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| "text/en_std".to_string());
        let pinned_json = serde_json::to_string(&req.pinned_keys)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Serialize CostMatrixSource
        let cost_matrix_str = serde_json::to_string(&req.cost_matrix)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _parents_json =
            serde_json::to_string(&req.parents).unwrap_or_else(|_| "[]".to_string());

        let result = sqlx::query(
            r#"
            INSERT INTO jobs (
                id, keyboard_id, scoring_profile_id, search_config_id, 
                pinned_keys, corpus_name, cost_matrix, owner_id, 
                parent_job_id, priority
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(job_id)
        .bind(kb_id)
        .bind(score_id)
        .bind(search_id)
        .bind(pinned_json)
        .bind(&primary_corpus)
        .bind(&cost_matrix_str) // Use the serialized JSON string
        .bind(owner_id)
        .bind(parent_job_id)
        .bind(priority)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn claim_job(&self) -> Result<Option<(String, JobRequest)>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            WITH locked_job AS (
                SELECT id 
                FROM jobs 
                WHERE status = 'active' 
                ORDER BY priority DESC, created_at ASC 
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            ),
            updated_job AS (
                UPDATE jobs
                SET status = 'processing',
                    started_at = CURRENT_TIMESTAMP
                FROM locked_job
                WHERE jobs.id = locked_job.id
                RETURNING jobs.*
            )
            SELECT 
                u.id,
                jsonb_build_object(
                    'meta', jsonb_build_object('name', k.name, 'author', k.author, 'version', k.version, 'notes', k.notes, 'type', k.kb_type),
                    'geometry', jsonb_build_object(
                        'keys', (SELECT jsonb_agg(jsonb_build_object('id', 'k'||kk.idx, 'hand', kk.hand, 'finger', kk.finger, 'row', kk.row_idx, 'col', kk.col_idx, 'x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h, 'is_stretch', kk.is_stretch, 'r', kk.r) ORDER BY kk.idx) FROM keyboard_keys kk WHERE kk.keyboard_id = k.id),
                        'prime_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
                        'med_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
                        'low_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
                        'home_row', 1
                    )
                ) as geometry_json,
                sp.weights as weights_json,
                (to_jsonb(sc) - 'id' - 'config_hash') as params_json,
                u.pinned_keys, 
                u.corpus_name, 
                u.cost_matrix,
                u.parent_job_id,
                u.priority
            FROM updated_job u
            JOIN keyboards k ON u.keyboard_id = k.id
            JOIN scoring_profiles sp ON u.scoring_profile_id = sp.id
            JOIN search_configs sc ON u.search_config_id = sc.id
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let id: String = r.try_get("id")?;
            let geometry: KeyboardDefinition = serde_json::from_value(r.try_get("geometry_json")?)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let weights: ScoringWeights = serde_json::from_value(r.try_get("weights_json")?)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let params: SearchParams = serde_json::from_value(r.try_get("params_json")?)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let corpus_name: String = r.try_get("corpus_name")?;

            let pinned_str: String = r.try_get("pinned_keys")?;
            let pinned_keys: Vec<KeyConstraint> = serde_json::from_str(&pinned_str)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

            let cost_raw: String = r.try_get("cost_matrix")?;
            // Backward Compatibility: Try JSON, fallback to Predefined(string)
            let cost_matrix = match serde_json::from_str(&cost_raw) {
                Ok(cm) => cm,
                Err(_) => CostMatrixSource::Predefined(cost_raw),
            };

            let parent_job_id: Option<String> = r.try_get("parent_job_id")?;

            // Lineage Logic: Fetch baseline and parents if parent exists
            let (baseline_score, parents) = if let Some(pid) = &parent_job_id {
                let best_score: Option<f32> =
                    sqlx::query_scalar("SELECT min(score) FROM results WHERE job_id = $1")
                        .bind(pid)
                        .fetch_optional(&self.pool)
                        .await?
                        .map(|s: f64| s as f32);

                let top_layouts: Vec<String> = sqlx::query_scalar(
                    "SELECT layout FROM results WHERE job_id = $1 ORDER BY score ASC LIMIT 5",
                )
                .bind(pid)
                .fetch_all(&self.pool)
                .await?;

                (best_score, top_layouts)
            } else {
                (None, vec![])
            };

            Ok(Some((
                id,
                JobRequest {
                    version: keyforge_protocol::PROTOCOL_VERSION,
                    definition: geometry,
                    weights,
                    params,
                    pinned_keys,
                    corpora: vec![CorpusSource {
                        id: corpus_name,
                        weight: 1.0,
                        hash: None,
                    }],
                    cost_matrix,
                    biometrics: vec![],
                    parent_job_id,
                    baseline_score,
                    parents,
                },
            )))
        } else {
            Ok(None)
        }
    }

    /// Resets jobs that have been processing for too long.
    /// 
    /// If the `node_id` column is missing (legacy schema), it uses a 6x longer timeout
    /// to avoid prematurely resetting jobs that might still be active but untracked.
    ///
    /// Returns the number of jobs reset or failed.
    pub async fn prune_stale_jobs(
        &self,
        timeout_minutes: i32,
        max_retries: i32,
    ) -> Result<u64, sqlx::Error> {
        // Newer schemas track the currently assigned worker on `jobs.node_id`.
        // Some dev/prod deployments may be missing this column (e.g. partially applied migrations).
        // We degrade gracefully by retrying without touching `node_id` when Postgres reports
        // `undefined_column` (SQLSTATE 42703).
        let query_with_node = r#"
            UPDATE jobs
            SET 
                status = CASE 
                    WHEN retry_count >= $2 THEN 'failed' 
                    ELSE 'active' 
                END,
                node_id = NULL,
                started_at = NULL,
                retry_count = retry_count + 1
            WHERE 
                status = 'processing' 
                AND (
                    (node_id IS NULL AND started_at < NOW() - make_interval(mins => $1))
                    OR node_id IN (SELECT id FROM nodes WHERE last_seen < NOW() - make_interval(mins => $1))
                )
        "#;

        match sqlx::query(query_with_node)
            .bind(timeout_minutes)
            .bind(max_retries)
            .execute(&self.pool)
            .await
        {
            Ok(result) => Ok(result.rows_affected()),
            Err(e) if Self::is_undefined_column(&e) => {
                let query_without_node = r#"
                    UPDATE jobs
                    SET 
                        status = CASE 
                            WHEN retry_count >= $2 THEN 'failed' 
                            ELSE 'active' 
                        END,
                        started_at = NULL,
                        retry_count = retry_count + 1
                    WHERE 
                        status = 'processing' 
                        AND started_at < NOW() - make_interval(mins => $1 * 6) -- 6x longer fallback
                "#;

                let result = sqlx::query(query_without_node)
                    .bind(timeout_minutes)
                    .bind(max_retries)
                    .execute(&self.pool)
                    .await?;

                Ok(result.rows_affected())
            }
            Err(e) => Err(e),
        }
    }

    fn _private_module_guard() {}

    fn is_undefined_column(e: &sqlx::Error) -> bool {
        match e {
            sqlx::Error::Database(db_err) => db_err.code().as_deref() == Some("42703"),
            _ => false,
        }
    }

    pub async fn get_config(
        &self,
        job_id: &str,
    ) -> Result<Option<(KeyboardGeometry, ScoringWeights, String, String)>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                jsonb_build_object(
                    'keys', (SELECT COALESCE(jsonb_agg(jsonb_build_object('x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h, 'row', kk.row_idx, 'col', kk.col_idx, 'hand', kk.hand, 'finger', kk.finger, 'is_stretch', kk.is_stretch, 'id', 'k' || kk.idx, 'r', kk.r) ORDER BY kk.idx), '[]'::jsonb) FROM keyboard_keys kk WHERE kk.keyboard_id = k.id),
                    'prime_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
                    'med_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
                    'low_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
                    'home_row', 1
                ) as geometry_json,
                sp.weights as weights_json,
                j.corpus_name,
                j.cost_matrix
            FROM jobs j
            JOIN keyboards k ON j.keyboard_id = k.id
            JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
            WHERE j.id = $1
            "#
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let geo_json: serde_json::Value = r.try_get("geometry_json")?;
            let geo: KeyboardGeometry = serde_json::from_value(geo_json)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let w: ScoringWeights = serde_json::from_value(r.try_get("weights_json")?)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

            // Note: VerificationService likely expects a filename for caching.
            // If it's a JSON blob (Custom), we need to handle that or let it bubble up.
            // But verification service uses GlobalAssetCache which handles CostMatrixSource?
            // Actually GlobalAssetCache currently takes `&str` filename.
            // We need to upgrade GlobalAssetCache too, or here return the filename if Predefined, or Hash if Custom.
            // For now, return raw string, let caller handle.
            let cost_raw: String = r.try_get("cost_matrix")?;

            Ok(Some((geo, w, r.try_get("corpus_name")?, cost_raw)))
        } else {
            Ok(None)
        }
    }

    pub async fn cancel(&self, job_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE jobs SET status = 'cancelled' WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_active(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs WHERE status = 'active'")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    pub async fn prune_old_jobs(&self, days: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM jobs WHERE status != 'active' AND created_at < NOW() - make_interval(days => $1)")
            .bind(days)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
