use super::queries;
use super::identity;
use keyforge_model::{CorpusSource, ScoringWeights, SearchParams, KeyIndex};
use keyforge_model::constants::{MAX_PINNED_KEYS_COUNT, DEFAULT_CORPUS_ID, DEFAULT_CORPUS_WEIGHT};
use keyforge_model::{KeyboardDefinition, KeyboardGeometry};
use keyforge_model::{CostMatrixSource, KeyConstraint, Validator};
use keyforge_protocol::JobRequest;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

/// Repository for managing job life cycles, registration, and claiming.
#[derive(Clone, Debug)]
pub struct JobRepository {
    /// The underlying Postgres connection pool.
    pub(crate) pool: Pool<Postgres>,
}

impl JobRepository {
    /// Creates a new `JobRepository` with the given connection pool.
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Checks if the database is reachable.
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Returns true if a job with the given ID already exists.
    pub async fn exists(&self, job_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("SELECT 1 FROM jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.is_some())
    }

    /// Registers a new optimization job in the database.
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

        self.validate_registration_request(req)?;

        // 1. Calculate deterministic component hashes (CPU-bound)
        let req_clone = req.clone();
        let (unique_hash, w_json, w_hash, p_hash) = tokio::task::spawn_blocking(move || {
            identity::calculate_job_identity(&req_clone)
        })
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("Hashing task failed: {}", e)))?
        .map_err(sqlx::Error::Protocol)?;

        let mut tx = self.pool.begin().await?;

        // 2. Advisory Lock
        self.acquire_advisory_lock(&mut tx, &unique_hash).await?;

        // 3. Ensure Components Exist
        let kb_id = self.ensure_keyboard(&mut tx, &req.config.definition, &unique_hash).await?;
        let score_id = self.ensure_scoring_profile(&mut tx, &w_json, &w_hash).await?;
        let search_id = self.ensure_search_config(&mut tx, &req.config.params, &p_hash).await?;

        // 4. Insert Job Record
        let is_new = self.insert_job_record(
            &mut tx, job_id, kb_id, score_id, search_id, req, owner_id, parent_job_id, priority
        ).await?;

        tx.commit().await?;

        Ok(is_new)
    }

    fn validate_registration_request(&self, req: &JobRequest) -> Result<(), sqlx::Error> {
        req.config.params
            .validate()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid search parameters: {}", e)))?;
        req.config.weights
            .validate()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid scoring weights: {}", e)))?;

        if req.config.pinned_keys.len() > MAX_PINNED_KEYS_COUNT {
            return Err(sqlx::Error::Protocol("Pinned keys too large".into()));
        }
        Ok(())
    }

    async fn insert_job_record(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        job_id: &str,
        kb_id: i32,
        score_id: i32,
        search_id: i32,
        req: &JobRequest,
        owner_id: Option<Uuid>,
        parent_job_id: Option<String>,
        priority: i32,
    ) -> Result<bool, sqlx::Error> {
        let primary_corpus = req
            .config.corpora
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| DEFAULT_CORPUS_ID.to_string());
        
        let result = sqlx::query(queries::INSERT_JOB_QUERY)
        .bind(job_id)
        .bind(kb_id)
        .bind(score_id)
        .bind(search_id)
        .bind(serde_json::to_string(&req.config.pinned_keys).unwrap_or_default())
        .bind(&primary_corpus)
        .bind(serde_json::to_string(&req.config.cost_matrix).unwrap_or_default())
        .bind(owner_id)
        .bind(parent_job_id)
        .bind(priority)
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn acquire_advisory_lock(&self, tx: &mut sqlx::Transaction<'_, Postgres>, unique_hash: &str) -> Result<(), sqlx::Error> {
        let mut bytes = [0u8; 8];
        let hash_bytes = hex::decode(unique_hash).unwrap_or_default();
        if hash_bytes.len() >= 8 {
            bytes.copy_from_slice(&hash_bytes[0..8]);
        }
        let lock_id = i64::from_be_bytes(bytes);

        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(lock_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn ensure_keyboard(
        &self, 
        tx: &mut sqlx::Transaction<'_, Postgres>, 
        def: &KeyboardDefinition, 
        unique_hash: &str
    ) -> Result<i32, sqlx::Error> {
        let kb_meta = &def.meta;
        let row = sqlx::query(queries::INSERT_KEYBOARD_QUERY)
        .bind(&kb_meta.name)
        .bind(&kb_meta.author)
        .bind(&kb_meta.version)
        .bind(&kb_meta.notes)
        .bind(&kb_meta.kb_type)
        .bind(unique_hash)
        .fetch_one(&mut **tx)
        .await?;

        let kb_id: i32 = row.try_get("id")?;

        let keys_exist = sqlx::query("SELECT 1 FROM keyboard_keys WHERE keyboard_id = $1 LIMIT 1")
                .bind(kb_id)
                .fetch_optional(&mut **tx)
                .await?;

        if keys_exist.is_none() {
            for (idx, key) in def.geometry.keys.iter().enumerate() {
                let kidx = KeyIndex(idx as u16);
                let is_prime = def.geometry.prime_slots.contains(&kidx);
                let is_med = def.geometry.med_slots.contains(&kidx);
                let is_low = def.geometry.low_slots.contains(&kidx);

                sqlx::query(queries::INSERT_KEY_QUERY)
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
                .execute(&mut **tx)
                .await?;
            }
        }
        Ok(kb_id)
    }

    async fn ensure_scoring_profile(
        &self, 
        tx: &mut sqlx::Transaction<'_, Postgres>, 
        w_json: &serde_json::Value,
        w_hash: &str
    ) -> Result<i32, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO scoring_profiles (weights, config_hash) 
            VALUES ($1, $2)
            ON CONFLICT (config_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
        )
        .bind(w_json)
        .bind(w_hash)
        .fetch_one(&mut **tx)
        .await?;
        Ok(row.try_get("id")?)
    }

    async fn ensure_search_config(
        &self, 
        tx: &mut sqlx::Transaction<'_, Postgres>, 
        params: &SearchParams, 
        p_hash: &str
    ) -> Result<i32, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO search_configs (
                search_epochs, search_steps, search_patience, search_patience_threshold,
                temp_min, temp_max, opt_limit_fast, opt_limit_slow, config_hash
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (config_hash) DO UPDATE SET id = search_configs.id
            RETURNING id
            "#,
        )
        .bind(params.get_search_epochs() as i32)
        .bind(params.get_search_steps() as i32)
        .bind(params.get_search_patience() as i32)
        .bind(params.get_search_patience_threshold())
        .bind(params.get_temp_min())
        .bind(params.get_temp_max())
        .bind(params.get_opt_limit_fast() as i32)
        .bind(params.get_opt_limit_slow() as i32)
        .bind(p_hash)
        .fetch_one(&mut **tx)
        .await?;
        Ok(row.try_get("id")?)
    }

    /// Attempts to claim an 'active' job for a worker node.
    pub async fn claim_job(&self) -> Result<Option<(String, JobRequest)>, sqlx::Error> {
        let row = sqlx::query(queries::CLAIM_JOB_QUERY)
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
            let cost_matrix = match serde_json::from_str(&cost_raw) {
                Ok(cm) => cm,
                Err(_) => CostMatrixSource::Predefined(cost_raw),
            };

            let parent_job_id: Option<String> = r.try_get("parent_job_id")?;

            // Lineage Logic
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
                    config: keyforge_protocol::JobConfig {
                        definition: geometry,
                        weights,
                        params,
                        pinned_keys,
                        corpora: vec![CorpusSource {
                            id: corpus_name,
                            weight: DEFAULT_CORPUS_WEIGHT,
                            hash: None,
                        }],
                        cost_matrix,
                        biometrics: vec![],
                        parent_job_id,
                        baseline_score,
                        parents,
                    },
                },
            )))
        } else {
            Ok(None)
        }
    }

    /// Resets jobs that have been processing for too long.
    pub async fn prune_stale_jobs(
        &self,
        timeout_minutes: i32,
        max_retries: i32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(queries::PRUNE_STALE_JOBS_WITH_NODE)
            .bind(timeout_minutes)
            .bind(max_retries)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected())
    }

    /// Retrieves the configuration required to verify or re-run a job.
    pub async fn get_config(
        &self,
        job_id: &str,
    ) -> Result<Option<(KeyboardGeometry, ScoringWeights, String, String)>, sqlx::Error> {
        let row = sqlx::query(queries::GET_JOB_CONFIG_QUERY)
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let geo_json: serde_json::Value = r.try_get("geometry_json")?;
            let geo: KeyboardGeometry = serde_json::from_value(geo_json)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let w: ScoringWeights = serde_json::from_value(r.try_get("weights_json")?)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let cost_raw: String = r.try_get("cost_matrix")?;

            Ok(Some((geo, w, r.try_get("corpus_name")?, cost_raw)))
        } else {
            Ok(None)
        }
    }

    /// Transitions a job to the 'cancelled' state.
    pub async fn cancel(&self, job_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE jobs SET status = 'cancelled' WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Returns the number of jobs currently in the 'active' state.
    pub async fn count_active(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs WHERE status = 'active'")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Deletes jobs that are no longer active and are older than the specified duration.
    pub async fn prune_old_jobs(&self, days: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM jobs WHERE status != 'active' AND created_at < NOW() - make_interval(days => $1)")
            .bind(days)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
