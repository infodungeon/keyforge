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

    /// Registers a job using the `register_full_job` Stored Procedure.
    /// This normalizes the keyboard, weights, and params into their respective tables.
    pub async fn register_job(&self, job_id: &str, req: &RegisterJobRequest) -> Result<(), String> {
        // 1. Validation
        if req.pinned_keys.len() > MAX_TEXT_LEN {
            return Err("Pinned keys configuration too large".into());
        }

        // 2. Serialize Components to JSONB Values
        // req.definition contains .meta and .geometry
        let meta_json = serde_json::to_value(&req.definition.meta).map_err(|e| e.to_string())?;

        // We strip the geometry down to keys for the SP array input
        let keys_json =
            serde_json::to_value(&req.definition.geometry.keys).map_err(|e| e.to_string())?;

        // Construct the slots object explicitly
        let slots_json = serde_json::json!({
            "prime_slots": req.definition.geometry.prime_slots,
            "med_slots": req.definition.geometry.med_slots,
            "low_slots": req.definition.geometry.low_slots
        });

        let weights_json = serde_json::to_value(&req.weights).map_err(|e| e.to_string())?;

        // FIXED: Removed '&' before req.params (Clippy fix)
        let params_json = serde_json::to_value(req.params).map_err(|e| e.to_string())?;

        // 3. Call Stored Procedure
        // register_full_job(job_id, meta, keys, slots, weights, params, pinned, corpus, cost)
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

    /// Fetches the most recent active job from the `v_active_jobs` view.
    pub async fn get_latest_job(&self) -> Result<Option<(String, RegisterJobRequest)>, String> {
        // The view v_active_jobs reconstructs the JSONB objects we need
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

            // Deserialize normalized data back into Rust structs
            // Note: The view constructs a partial KeyboardDefinition structure in geometry_json
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

    /// Registers a Worker Node and its Hardware capabilities.
    pub async fn register_node_hardware(
        &self,
        node_id: &str,
        cpu_model: &str,
        cores: i32,
        l2_cache_kb: Option<i32>,
        ops_per_sec: f32,
    ) -> Result<(), String> {
        // 1. Upsert Hardware Profile using Stored Procedure
        sqlx::query("SELECT register_node_heartbeat($1, $2, $3, $4, $5, $6)")
            .bind(node_id)
            .bind(cpu_model)
            .bind(std::env::consts::ARCH)
            .bind(cores)
            .bind(l2_cache_kb)
            .bind(ops_per_sec as f64)
            .execute(&self.db)
            .await
            .map_err(|e| format!("Node Heartbeat Error: {}", e))?;

        Ok(())
    }

    /// Retrieves configuration for a specific job ID.
    pub async fn get_job_config(
        &self,
        job_id: &str,
    ) -> Result<Option<(KeyboardGeometry, ScoringWeights, String, String)>, String> {
        // We reconstruct the JSONB manually here to avoid relying on v_active_jobs filter
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
                    'prime_slots', (SELECT jsonb_agg(idx) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
                    'med_slots', (SELECT jsonb_agg(idx) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
                    'low_slots', (SELECT jsonb_agg(idx) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
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
            // Note: We only need Geometry for the worker to process results, not full definition
            let geo: KeyboardGeometry = serde_json::from_value(r.get("geometry_json"))
                .map_err(|e| format!("Corrupt Geometry: {}", e))?;

            let w: ScoringWeights = serde_json::from_value(r.get("weights_json"))
                .map_err(|e| format!("Corrupt Weights: {}", e))?;

            Ok(Some((geo, w, r.get("corpus_name"), r.get("cost_matrix"))))
        } else {
            Ok(None)
        }
    }

    pub async fn get_job_population(&self, job_id: &str) -> Result<Vec<String>, String> {
        let rows = sqlx::query(
            "SELECT layout FROM results WHERE job_id = $1 GROUP BY layout ORDER BY score ASC LIMIT 50"
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
