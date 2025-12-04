use crate::routes::jobs::RegisterJobRequest;
use crate::routes::submission::SubmissionEntry;
use keyforge_core::{config::ScoringWeights, geometry::KeyboardGeometry};
use sqlx::{Pool, Row, Sqlite, Transaction};

#[derive(Clone)]
pub struct Store {
    pub db: Pool<Sqlite>,
}

const MAX_TEXT_LEN: usize = 10_000;

impl Store {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    // FIXED: Suppress warning as Queue uses batching now instead of explicit transactions
    #[allow(dead_code)]
    pub async fn begin(&self) -> Result<Transaction<'_, Sqlite>, String> {
        self.db.begin().await.map_err(|e| e.to_string())
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
        if req.pinned_keys.len() > MAX_TEXT_LEN {
            return Err("Pinned keys configuration too large".into());
        }

        let geo_json = serde_json::to_string(&req.geometry).map_err(|e| e.to_string())?;
        if geo_json.len() > MAX_TEXT_LEN * 10 {
            return Err("Geometry configuration too large".into());
        }

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

    pub async fn save_submission(
        &self,
        name: &str,
        layout: &str,
        author: &str,
    ) -> Result<i64, String> {
        if name.len() > 100 {
            return Err("Name too long".into());
        }
        if author.len() > 100 {
            return Err("Author name too long".into());
        }
        if layout.len() > MAX_TEXT_LEN {
            return Err("Layout data too long".into());
        }

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

    pub async fn get_recent_submissions(&self, limit: i64) -> Result<Vec<SubmissionEntry>, String> {
        let rows = sqlx::query(
            "SELECT id, name, layout_str, author, submitted_at FROM submissions ORDER BY submitted_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        let entries = rows
            .into_iter()
            .map(|r| SubmissionEntry {
                id: r.get("id"),
                name: r.get("name"),
                layout: r.get("layout_str"),
                author: r.get("author"),
                date: r.get("submitted_at"),
            })
            .collect();

        Ok(entries)
    }
}
