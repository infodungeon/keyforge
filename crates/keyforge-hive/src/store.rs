// ===== keyforge/crates/keyforge-hive/src/store.rs =====
use crate::routes::submission::SubmissionEntry;
use keyforge_core::config::{ScoringWeights, SearchParams};
use keyforge_core::geometry::{KeyboardDefinition, KeyboardGeometry};
use keyforge_core::protocol::RegisterJobRequest;
use sqlx::{Pool, Postgres, Row};

#[derive(Clone)]
pub struct Store {
    pub db: Pool<Postgres>,
}

const MAX_TEXT_LEN: usize = 50_000;

impl Store {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn job_exists(&self, job_id: &str) -> bool {
        let result = sqlx::query("SELECT 1 FROM jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&self.db)
            .await
            .unwrap_or(None);
        result.is_some()
    }

    pub async fn register_job(&self, job_id: &str, req: &RegisterJobRequest) -> Result<(), String> {
        if req.pinned_keys.len() > MAX_TEXT_LEN {
            return Err("Pinned keys configuration too large".into());
        }

        let meta_json = serde_json::to_value(&req.definition.meta).map_err(|e| e.to_string())?;
        let keys_json =
            serde_json::to_value(&req.definition.geometry.keys).map_err(|e| e.to_string())?;

        let slots_json = serde_json::json!({
            "prime_slots": req.definition.geometry.prime_slots,
            "med_slots": req.definition.geometry.med_slots,
            "low_slots": req.definition.geometry.low_slots
        });

        let weights_json = serde_json::to_value(&req.weights).map_err(|e| e.to_string())?;
        let params_json = serde_json::to_value(req.params).map_err(|e| e.to_string())?;

        sqlx::query("SELECT register_full_job($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(job_id)
            .bind(meta_json)
            .bind(keys_json)
            .bind(slots_json)
            .bind(weights_json)
            .bind(params_json)
            .bind(&req.pinned_keys)
            .bind(&req.corpus_name)
            .bind(&req.cost_matrix)
            .execute(&self.db)
            .await
            .map_err(|e| format!("Database Error (SP): {}", e))?;

        Ok(())
    }

    pub async fn get_latest_job(&self) -> Result<Option<(String, RegisterJobRequest)>, String> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, geometry_json, weights_json, params_json, 
                pinned_keys, corpus_name, cost_matrix 
            FROM v_active_jobs 
            ORDER BY created_at DESC 
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            let id: String = r.get("id");
            let definition: KeyboardDefinition = serde_json::from_value(r.get("geometry_json"))
                .map_err(|e| format!("Corrupt Geometry JSON: {}", e))?;
            let weights: ScoringWeights = serde_json::from_value(r.get("weights_json"))
                .map_err(|e| format!("Corrupt Weights JSON: {}", e))?;
            let params: SearchParams = serde_json::from_value(r.get("params_json"))
                .map_err(|e| format!("Corrupt Params JSON: {}", e))?;

            Ok(Some((
                id,
                RegisterJobRequest {
                    definition,
                    weights,
                    params,
                    pinned_keys: r.get("pinned_keys"),
                    corpus_name: r.get("corpus_name"),
                    cost_matrix: r.get("cost_matrix"),
                },
            )))
        } else {
            Ok(None)
        }
    }

    pub async fn register_node_hardware(
        &self,
        node_id: &str,
        cpu_model: &str,
        cores: i32,
        l2_cache_kb: Option<i32>,
        ops_per_sec: f32,
    ) -> Result<(), String> {
        // F32 -> REAL binding (Matched to schema.sql)
        sqlx::query("SELECT register_node_heartbeat($1, $2, $3, $4, $5, $6)")
            .bind(node_id)
            .bind(cpu_model)
            .bind(std::env::consts::ARCH)
            .bind(cores)
            .bind(l2_cache_kb)
            .bind(ops_per_sec)
            .execute(&self.db)
            .await
            .map_err(|e| format!("Node Heartbeat Error: {}", e))?;

        Ok(())
    }

    pub async fn get_job_config(
        &self,
        job_id: &str,
    ) -> Result<Option<(KeyboardGeometry, ScoringWeights, String, String)>, String> {
        let row = sqlx::query(
            r#"
            SELECT 
                jsonb_build_object(
                    'keys', (
                        SELECT jsonb_agg(jsonb_build_object(
                            'x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h,
                            'row', kk.row_idx, 'col', kk.col_idx, 
                            'hand', kk.hand, 'finger', kk.finger,
                            'is_stretch', kk.is_stretch, 'id', 'k' || kk.idx
                        ) ORDER BY kk.idx)
                        FROM keyboard_keys kk WHERE kk.keyboard_id = k.id
                    ),
                    'prime_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
                    'med_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
                    'low_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
                    'home_row', 1
                ) as geometry_json,
                to_jsonb(sp) - 'id' - 'config_hash' - 'created_at' as weights_json,
                j.corpus_name,
                j.cost_matrix
            FROM jobs j
            JOIN keyboards k ON j.keyboard_id = k.id
            JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
            WHERE j.id = $1
            "#
        )
        .bind(job_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            let geo: KeyboardGeometry = serde_json::from_value(r.get("geometry_json"))
                .map_err(|e| format!("Corrupt Geometry: {}", e))?;

            let w: ScoringWeights = serde_json::from_value(r.get("weights_json"))
                .map_err(|e| format!("Corrupt Weights: {}", e))?;

            Ok(Some((geo, w, r.get("corpus_name"), r.get("cost_matrix"))))
        } else {
            Ok(None)
        }
    }

    /// Returns a population of layouts for a worker to seed from.
    /// IMPLEMENTS CROSS-POLLINATION:
    /// 1. Top 45 layouts from the current job (Evolution).
    /// 2. Top 5 layouts from ANY job that uses the same Keyboard Geometry (Migration).
    pub async fn get_job_population(&self, job_id: &str) -> Result<Vec<String>, String> {
        // We use UNION ALL to combine the local gene pool with the alien gene pool
        let rows = sqlx::query(
            r#"
            (SELECT layout, MIN(score) as s FROM results 
             WHERE job_id = $1 
             GROUP BY layout 
             ORDER BY s ASC 
             LIMIT 45)
            UNION ALL
            (SELECT r.layout, MIN(r.score) as s FROM results r
             JOIN jobs j ON r.job_id = j.id
             WHERE j.keyboard_id = (SELECT keyboard_id FROM jobs WHERE id = $1)
             AND r.job_id != $1
             GROUP BY r.layout
             ORDER BY s ASC 
             LIMIT 5)
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows.iter().map(|r| r.get("layout")).collect())
    }

    pub async fn get_job_best_score(&self, job_id: &str) -> Result<Option<f32>, String> {
        let row = sqlx::query("SELECT min(score) as min_score FROM results WHERE job_id = $1")
            .bind(job_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(row
            .and_then(|r| r.get::<Option<f64>, _>("min_score"))
            .map(|v| v as f32))
    }

    pub async fn save_submission(
        &self,
        name: &str,
        layout: &str,
        author: &str,
    ) -> Result<i64, String> {
        let rec = sqlx::query(
            "INSERT INTO submissions (name, layout_str, author) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(name)
        .bind(layout)
        .bind(author)
        .fetch_one(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rec.get::<i32, _>("id") as i64)
    }

    pub async fn get_recent_submissions(&self, limit: i64) -> Result<Vec<SubmissionEntry>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, layout_str, author, submitted_at 
            FROM submissions 
            ORDER BY submitted_at DESC 
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|r| SubmissionEntry {
                id: r.get::<i32, _>("id") as i64,
                name: r.get("name"),
                layout: r.get("layout_str"),
                author: r.get("author"),
                date: r
                    .get::<chrono::DateTime<chrono::Utc>, _>("submitted_at")
                    .to_rfc3339(),
            })
            .collect())
    }
}
