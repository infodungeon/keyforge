// libs/keyforge-persistence/src/repo/community_repo.rs

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use crate::error::{PersistenceError, PersistenceResult};
use keyforge_model::community::LayoutSubmission;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for community layout submissions.
#[derive(Debug, Clone)]
pub struct CommunityRepository {
    pool: PgPool,
}

impl CommunityRepository {
    /// Creates a new `CommunityRepository`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Submits a new layout to the community repository.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the submission fails.
    pub async fn submit_layout(&self, submission: &LayoutSubmission) -> PersistenceResult<()> {
        let layout_json = serde_json::to_value(&submission.layout)?;

        sqlx::query!(
            r#"
            INSERT INTO layout_submissions (id, author_id, keyboard_id, layout_data, score, tags)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            Uuid::parse_str(&submission.id).unwrap_or_else(|_| Uuid::new_v4()),
            submission.author_id.as_uuid(),
            submission.keyboard_id.as_str(),
            layout_json,
            submission.score.as_fixed() as i32,
            &submission.tags
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    /// Retrieves recent community submissions.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the query fails.
    pub async fn get_recent_submissions(
        &self,
        limit: i64,
    ) -> PersistenceResult<Vec<LayoutSubmission>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, author_id, keyboard_id, layout_data, score, tags, created_at
            FROM layout_submissions
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        let mut submissions = Vec::new();
        for r in rows {
            submissions.push(LayoutSubmission {
                id: r.id.to_string(),
                author_id: r.author_id.into(),
                keyboard_id: r.keyboard_id.into(),
                layout: serde_json::from_value(r.layout_data)?,
                score: (r.score as u32).into(),
                tags: r.tags.unwrap_or_default(),
                created_at: r.created_at.map_or(0, |t| t.timestamp() as u64),
            });
        }

        Ok(submissions)
    }
}
