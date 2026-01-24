// apps/keyforge-hive/src/infra/repositories/jobs/core.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::identity;
use keyforge_model::config::ScoringWeights;
use keyforge_model::constants::MAX_PINNED_KEYS_COUNT;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::types::KeyIndex;
use keyforge_model::Validator;
use keyforge_protocol::JobRequest;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct JobRepository {
    pub pool: sqlx::PgPool,
}

impl JobRepository {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(
        &self,
        job_id: &str,
        req: &JobRequest,
        owner_id: Option<Uuid>,
        parent_id: Option<String>,
        priority: i32,
    ) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        Self::validate_registration_request(req)?;

        let req_clone = req.clone();
        let unique_hash =
            tokio::task::spawn_blocking(move || identity::calculate_job_identity(&req_clone))
                .await
                .map_err(|e| sqlx::Error::Protocol(format!("Hashing task failed: {e}")))?
                .map_err(sqlx::Error::Protocol)?;

        self.acquire_advisory_lock(&mut tx, &unique_hash).await?;

        let kb_id = self
            .ensure_keyboard(&mut tx, &req.config.definition, &unique_hash)
            .await?;
        let score_id = self
            .ensure_scoring_weights(&mut tx, &req.config.weights)
            .await?;
        let search_id = self
            .ensure_search_params(&mut tx, &req.config.params)
            .await?;

        let is_new = self
            .insert_job_record(
                &mut tx, job_id, kb_id, score_id, search_id, req, owner_id, parent_id, priority,
            )
            .await?;

        tx.commit().await?;
        Ok(is_new)
    }

    pub async fn claim_job(&self) -> Result<Option<(String, JobRequest)>, sqlx::Error> {
        let r = sqlx::query!(
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
                RETURNING jobs.id, jobs.keyboard_id, jobs.pinned_keys, jobs.corpus_name, jobs.cost_matrix, jobs.parent_job_id
            )
            SELECT 
                j.id,
                j.keyboard_id,
                sp.weights as weights_json,
                (to_jsonb(sc) - 'id' - 'config_hash') as params_json,
                j.pinned_keys, 
                j.corpus_name, 
                j.cost_matrix,
                j.parent_job_id
            FROM updated_job u
            JOIN jobs j ON u.id = j.id
            JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
            JOIN search_configs sc ON j.search_config_id = sc.id
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = r {
            let id = r.id;
            let keyboard_id = r.keyboard_id;
            let weights_val = r.weights_json;
            let params_val = r.params_json.unwrap_or_default();
            let pinned_keys_str = r.pinned_keys;
            let corpus_name = r.corpus_name;
            let cost_matrix_str = r.cost_matrix;

            let definition = self.fetch_keyboard_definition(keyboard_id).await?;

            let weights = serde_json::from_value(weights_val)
                .map_err(|e| sqlx::Error::Protocol(format!("Invalid weights in DB: {e}")))?;
            let params = serde_json::from_value(params_val)
                .map_err(|e| sqlx::Error::Protocol(format!("Invalid params in DB: {e}")))?;
            let pinned_keys = serde_json::from_str(&pinned_keys_str)
                .map_err(|e| sqlx::Error::Protocol(format!("Invalid pins in DB: {e}")))?;
            let cost_matrix = serde_json::from_str(&cost_matrix_str)
                .map_err(|e| sqlx::Error::Protocol(format!("Invalid cost_matrix in DB: {e}")))?;

            let req = JobRequest {
                version: keyforge_protocol::PROTOCOL_VERSION,
                config: keyforge_protocol::JobConfig {
                    definition,
                    weights,
                    params,
                    pinned_keys,
                    corpora: vec![keyforge_model::CorpusSource {
                        id: corpus_name,
                        weight: keyforge_model::constants::DEFAULT_CORPUS_WEIGHT,
                        hash: None,
                    }],
                    cost_matrix,
                    biometrics: vec![],
                    parent_job_id: r.parent_job_id,
                    baseline_score: None,
                    parents: vec![],
                },
            };
            Ok(Some((id, req)))
        } else {
            Ok(None)
        }
    }

    pub async fn cancel(&self, job_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("UPDATE jobs SET status = 'cancelled' WHERE id = $1", job_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_active(&self) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM jobs WHERE status = 'active' OR status = 'processing'"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count.unwrap_or(0))
    }

    pub async fn get_config(
        &self,
        job_id: &str,
    ) -> Result<
        Option<(
            keyforge_model::geometry::KeyboardGeometry,
            ScoringWeights,
            String,
            keyforge_model::CostMatrixSource,
        )>,
        sqlx::Error,
    > {
        let row = sqlx::query!(
            r#"
            SELECT 
                j.keyboard_id,
                sp.weights as weights_json,
                j.corpus_name,
                j.cost_matrix
            FROM jobs j
            JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
            WHERE j.id = $1
            "#,
            job_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let keyboard_id = r.keyboard_id;
            let definition = self.fetch_keyboard_definition(keyboard_id).await?;
            let geometry = definition.geometry;

            let weights: ScoringWeights =
                serde_json::from_value(r.weights_json).unwrap_or_default();
            let corpus = r.corpus_name;
            let cost_raw = r.cost_matrix;
            let cost: keyforge_model::CostMatrixSource = serde_json::from_str(&cost_raw)
                .unwrap_or(keyforge_model::CostMatrixSource::Predefined(cost_raw));

            Ok(Some((geometry, weights, corpus, cost)))
        } else {
            Ok(None)
        }
    }

    async fn fetch_keyboard_definition(
        &self,
        kb_id: i32,
    ) -> Result<KeyboardDefinition, sqlx::Error> {
        let meta_row = sqlx::query!(
            r#"
            SELECT name, author, version, notes, kb_type, home_row 
            FROM keyboards 
            WHERE id = $1
            "#,
            kb_id
        )
        .fetch_one(&self.pool)
        .await?;

        let meta = keyforge_model::geometry::KeyboardMeta {
            name: meta_row.name,
            author: meta_row.author.unwrap_or_default(),
            version: meta_row.version.unwrap_or_default(),
            notes: meta_row.notes.unwrap_or_default(),
            kb_type: meta_row.kb_type.unwrap_or_default(),
        };
        let home_row = meta_row.home_row.unwrap_or(0);

        let keys_rows = sqlx::query!(
            r#"
            SELECT idx, x, y, w, h, hand, finger, row_idx, col_idx, is_stretch, is_prime, is_med, is_low, r 
            FROM keyboard_keys 
            WHERE keyboard_id = $1 
            ORDER BY idx
            "#,
            kb_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut keys = Vec::with_capacity(keys_rows.len());
        let mut prime_slots = Vec::new();
        let mut med_slots = Vec::new();
        let mut low_slots = Vec::new();

        for row in keys_rows {
            let idx = row.idx;
            #[allow(clippy::cast_possible_truncation)]
            let kidx = KeyIndex(
                u16::try_from(idx)
                    .map_err(|e| sqlx::Error::Protocol(format!("Key index overflow: {e}")))?,
            );

            keys.push(keyforge_model::KeyNode {
                index: usize::try_from(idx)
                    .map_err(|e| sqlx::Error::Protocol(format!("Key index negative: {e}")))?,
                label: format!("k{idx}"),
                x: row.x,
                y: row.y,
                w: row.w.unwrap_or(1.0),
                h: row.h.unwrap_or(1.0),
                hand: keyforge_model::types::HandIndex(u8::try_from(row.hand).unwrap_or(0)),
                finger: keyforge_model::types::FingerIndex::new_unchecked(
                    u8::try_from(row.finger).unwrap_or(0),
                ),
                row: keyforge_model::types::RowIndex(i8::try_from(row.row_idx).unwrap_or(0)),
                col: keyforge_model::types::ColIndex(i8::try_from(row.col_idx).unwrap_or(0)),
                is_stretch: row.is_stretch.unwrap_or(false),
                r: row.r.unwrap_or(0.0),
                ..Default::default()
            });

            if row.is_prime.unwrap_or(false) {
                prime_slots.push(kidx);
            }
            if row.is_med.unwrap_or(false) {
                med_slots.push(kidx);
            }
            if row.is_low.unwrap_or(false) {
                low_slots.push(kidx);
            }
        }

        Ok(KeyboardDefinition {
            meta,
            geometry: keyforge_model::geometry::KeyboardGeometry {
                keys,
                prime_slots,
                med_slots,
                low_slots,
                home_row: i8::try_from(home_row).unwrap_or(0),
            },
            layouts: std::collections::HashMap::new(),
        })
    }

    fn validate_registration_request(req: &JobRequest) -> Result<(), sqlx::Error> {
        req.config
            .params
            .validate()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid search parameters: {e}")))?;
        req.config
            .weights
            .validate()
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid scoring weights: {e}")))?;

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
        let primary_corpus = req.config.corpora.first().map_or_else(
            || keyforge_model::constants::DEFAULT_CORPUS_ID.to_string(),
            |c| c.id.clone(),
        );

        let result = sqlx::query!(
            r#"
            INSERT INTO jobs (
                id, keyboard_id, scoring_profile_id, search_config_id, 
                pinned_keys, corpus_name, cost_matrix, owner_id, 
                parent_job_id, priority
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO NOTHING
            "#,
            job_id,
            kb_id,
            score_id,
            search_id,
            serde_json::to_string(&req.config.pinned_keys).unwrap_or_default(),
            primary_corpus,
            serde_json::to_string(&req.config.cost_matrix).unwrap_or_default(),
            owner_id,
            parent_job_id,
            priority
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn acquire_advisory_lock(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        unique_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let mut bytes = [0u8; 8];
        let hash_bytes = hex::decode(unique_hash).unwrap_or_default();
        if hash_bytes.len() >= 8 {
            bytes.copy_from_slice(&hash_bytes[0..8]);
        }
        let lock_id = i64::from_be_bytes(bytes);

        sqlx::query!("SELECT pg_advisory_xact_lock($1)", lock_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn ensure_keyboard(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        def: &KeyboardDefinition,
        unique_hash: &str,
    ) -> Result<i32, sqlx::Error> {
        let kb_meta = &def.meta;
        let row = sqlx::query!(
            r#"
            INSERT INTO keyboards (name, author, version, notes, kb_type, home_row, unique_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (unique_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
            kb_meta.name,
            kb_meta.author,
            kb_meta.version,
            kb_meta.notes,
            kb_meta.kb_type,
            i32::from(def.geometry.home_row),
            unique_hash
        )
        .fetch_one(&mut **tx)
        .await?;

        let kb_id = row.id;

        let keys_exist = sqlx::query_scalar!(
            "SELECT 1 as one FROM keyboard_keys WHERE keyboard_id = $1 LIMIT 1",
            kb_id
        )
        .fetch_optional(&mut **tx)
        .await?;

        if keys_exist.is_none() {
            let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
                "INSERT INTO keyboard_keys (keyboard_id, idx, x, y, w, h, hand, finger, row_idx, col_idx, is_stretch, is_prime, is_med, is_low, r) "
            );

            qb.push_values(def.geometry.keys.iter().enumerate(), |mut b, (idx, key)| {
                #[allow(clippy::cast_possible_truncation)]
                let kidx = KeyIndex(idx as u16);
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let idx_i32 = idx as i32;

                b.push_bind(kb_id)
                    .push_bind(idx_i32)
                    .push_bind(key.x)
                    .push_bind(key.y)
                    .push_bind(key.w)
                    .push_bind(key.h)
                    .push_bind(i16::from(key.hand.0))
                    .push_bind(i16::from(key.finger.0))
                    .push_bind(i16::from(key.row.0))
                    .push_bind(i16::from(key.col.0))
                    .push_bind(key.is_stretch)
                    .push_bind(def.geometry.prime_slots.contains(&kidx))
                    .push_bind(def.geometry.med_slots.contains(&kidx))
                    .push_bind(def.geometry.low_slots.contains(&kidx))
                    .push_bind(key.r);
            });

            qb.build().execute(&mut **tx).await?;
        }

        Ok(kb_id)
    }

    async fn ensure_scoring_weights(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        weights: &keyforge_model::config::ScoringWeights,
    ) -> Result<i32, sqlx::Error> {
        let w_json = serde_json::to_string(weights).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(w_json.as_bytes());
        let hash = hex::encode(hasher.finalize());

        let row = sqlx::query!(
            r#"
            INSERT INTO scoring_profiles (config_hash, weights) 
            VALUES ($1, $2) 
            ON CONFLICT (config_hash) 
            DO UPDATE SET created_at = CURRENT_TIMESTAMP 
            RETURNING id
            "#,
            hash,
            serde_json::to_value(weights).unwrap_or_default()
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(row.id)
    }

    async fn ensure_search_params(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        params: &keyforge_model::config::SearchParams,
    ) -> Result<i32, sqlx::Error> {
        let p_json = serde_json::to_string(params).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(p_json.as_bytes());
        let hash = hex::encode(hasher.finalize());

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let epochs = params.get_search_epochs() as i32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let steps = params.get_search_steps() as i32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let patience = params.get_search_patience() as i32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let opt_fast = params.get_opt_limit_fast() as i32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let opt_slow = params.get_opt_limit_slow() as i32;

        let row = sqlx::query!(
            r#"
            INSERT INTO search_configs (
                config_hash, search_epochs, search_steps, search_patience, 
                search_patience_threshold, temp_min, temp_max, opt_limit_fast, opt_limit_slow
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
            ON CONFLICT (config_hash) 
            DO UPDATE SET config_hash = EXCLUDED.config_hash 
            RETURNING id
            "#,
            hash,
            epochs,
            steps,
            patience,
            params.get_search_patience_threshold(),
            params.get_temp_min(),
            params.get_temp_max(),
            opt_fast,
            opt_slow
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(row.id)
    }

    pub async fn prune_stale_jobs(
        &self,
        timeout_mins: i32,
        max_retries: i32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
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
            "#,
            timeout_mins,
            max_retries
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn prune_old_jobs(&self, days: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE jobs SET status = 'cancelled' WHERE status = 'active' AND created_at < NOW() - make_interval(days => $1)",
            days
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
