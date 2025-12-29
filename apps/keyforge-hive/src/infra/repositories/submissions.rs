// crates/keyforge-hive/src/repositories/submissions.rs
use crate::features::list_submissions::SubmissionEntry;
use sqlx::{Pool, Postgres, Row};

#[derive(Clone)]
pub struct SubmissionRepository {
    pool: Pool<Postgres>,
}

impl SubmissionRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save(&self, name: &str, layout: &str, author: &str) -> Result<i64, sqlx::Error> {
        let rec = sqlx::query(
            "INSERT INTO submissions (name, layout_str, author) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(name)
        .bind(layout)
        .bind(author)
        .fetch_one(&self.pool)
        .await?;

        let id: i32 = rec.try_get("id")?;
        Ok(id as i64)
    }

    pub async fn get_recent(&self, limit: i64) -> Result<Vec<SubmissionEntry>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, layout_str, author, submitted_at 
            FROM submissions 
            ORDER BY submitted_at DESC 
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SubmissionEntry {
                id: r.try_get::<i32, _>("id").unwrap_or(0) as i64,
                name: r.try_get("name").unwrap_or_default(),
                layout: r.try_get("layout_str").unwrap_or_default(),
                author: r.try_get("author").unwrap_or_default(),
                date: r
                    .try_get::<chrono::DateTime<chrono::Utc>, _>("submitted_at")
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_default(),
            })
            .collect())
    }
}
