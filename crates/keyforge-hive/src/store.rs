use crate::routes::jobs::RegisterJobRequest;
use keyforge_core::{config::ScoringWeights, geometry::KeyboardGeometry};
use sqlx::{Pool, Row, Sqlite};

#[derive(Clone)]
pub struct Store {
    db: Pool<Sqlite>,
}

impl Store {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    pub async fn job_exists(&self, job_id: &str) -> bool {
        sqlx::query("SELECT 1 FROM jobs WHERE id = ?")
            .bind(job_id)
            .fetch_optional(&self.db)
            .await
            .unwrap_or(None)
            .is_some()
    }

    pub async fn register_job(&self, job_id: &str, req: &RegisterJobRequest) -> Result<(), String> {
        let geo_json = serde_json::to_string(&req.geometry).map_err(|e| e.to_string())?;
        let weights_json = serde_json::to_string(&req.weights).map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO jobs (id, geometry_json, weights_json, pinned_keys, corpus_name) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(job_id)
        .bind(geo_json)
        .bind(weights_json)
        .bind(&req.pinned_keys)
        .bind(&req.corpus_name)
        .execute(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_latest_job(&self) -> Result<Option<(String, RegisterJobRequest)>, String> {
        let row_result = sqlx::query!(
            "SELECT id, geometry_json, weights_json, pinned_keys, corpus_name FROM jobs ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(r) = row_result {
            let geometry = serde_json::from_str::<KeyboardGeometry>(&r.geometry_json)
                .map_err(|e| format!("Corrupt Geometry: {}", e))?;

            let weights = serde_json::from_str::<ScoringWeights>(&r.weights_json)
                .map_err(|e| format!("Corrupt Weights: {}", e))?;

            // Normalize ID from SQLx inference
            let id = r.id.ok_or("Missing ID in DB")?;

            Ok(Some((
                id,
                RegisterJobRequest {
                    geometry,
                    weights,
                    pinned_keys: r.pinned_keys,
                    corpus_name: r.corpus_name,
                },
            )))
        } else {
            Ok(None)
        }
    }

    pub async fn get_job_config(
        &self,
        job_id: &str,
    ) -> Result<Option<(KeyboardGeometry, ScoringWeights, String)>, String> {
        let row =
            sqlx::query("SELECT geometry_json, weights_json, corpus_name FROM jobs WHERE id = ?")
                .bind(job_id)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            let g_str: String = r.get("geometry_json");
            let w_str: String = r.get("weights_json");
            let corpus: String = r.get("corpus_name");

            let geo = serde_json::from_str(&g_str).map_err(|e| e.to_string())?;
            let weights = serde_json::from_str(&w_str).map_err(|e| e.to_string())?;

            Ok(Some((geo, weights, corpus)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_job_population(&self, job_id: &str) -> Result<Vec<String>, String> {
        let rows = sqlx::query(
            "SELECT layout FROM results WHERE job_id = ? GROUP BY layout ORDER BY score ASC LIMIT 50"
        )
        .bind(job_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows.iter().map(|r| r.get("layout")).collect())
    }

    pub async fn get_job_best_score(&self, job_id: &str) -> Result<Option<f32>, String> {
        let row = sqlx::query("SELECT min(score) as min_score FROM results WHERE job_id = ?")
            .bind(job_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(row.and_then(|r| r.get("min_score")))
    }

    pub async fn save_result(
        &self,
        job_id: &str,
        layout: &str,
        score: f32,
        node: &str,
    ) -> Result<(), String> {
        sqlx::query("INSERT INTO results (job_id, layout, score, node_id) VALUES (?, ?, ?, ?)")
            .bind(job_id)
            .bind(layout)
            .bind(score)
            .bind(node)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn save_submission(
        &self,
        name: &str,
        layout: &str,
        author: &str,
    ) -> Result<i64, String> {
        let res =
            sqlx::query("INSERT INTO submissions (name, layout_str, author) VALUES (?, ?, ?)")
                .bind(name)
                .bind(layout)
                .bind(author)
                .execute(&self.db)
                .await
                .map_err(|e| e.to_string())?;

        Ok(res.last_insert_rowid())
    }
}
