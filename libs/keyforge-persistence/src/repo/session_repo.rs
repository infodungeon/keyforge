// libs/keyforge-persistence/src/repo/session_repo.rs

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use crate::error::{PersistenceError, PersistenceResult};
use keyforge_model::community::{AnalysisSession, AnalysisSessionEntry};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing optimization and analysis sessions.
#[derive(Debug, Clone)]
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    /// Creates a new `SessionRepository` with the given database pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new analysis session.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the database query fails.
    pub async fn create_session(&self, session: &AnalysisSession) -> PersistenceResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO analysis_sessions (id, user_id, keyboard_id, corpus_id)
            VALUES ($1, $2, $3, $4)
            "#,
            Uuid::parse_str(&session.id).unwrap_or_else(|_| Uuid::new_v4()),
            session.user_id.as_uuid(),
            session.keyboard_id.as_str(),
            session.corpus_id.as_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        for entry in &session.history {
            sqlx::query!(
                r#"
                INSERT INTO analysis_session_history (session_id, layout_id, score, timestamp)
                VALUES ($1, $2, $3, $4)
                "#,
                Uuid::parse_str(&session.id).unwrap_or_else(|_| Uuid::new_v4()),
                entry.layout_id.as_str(),
                entry.score.as_fixed() as i32,
                chrono::DateTime::<chrono::Utc>::from_timestamp(entry.timestamp as i64, 0)
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    /// Retrieves a full analysis session with its history.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the database query fails.
    pub async fn get_session(
        &self,
        session_id: Uuid,
    ) -> PersistenceResult<Option<AnalysisSession>> {
        let header = sqlx::query!(
            "SELECT id, user_id, keyboard_id, corpus_id FROM analysis_sessions WHERE id = $1",
            session_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        let Some(h) = header else {
            return Ok(None);
        };

        let history_rows = sqlx::query!(
            "SELECT layout_id, score, timestamp FROM analysis_session_history WHERE session_id = $1 ORDER BY timestamp ASC",
            session_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        let history = history_rows
            .into_iter()
            .map(|r| AnalysisSessionEntry {
                layout_id: r.layout_id.into(),
                score: (r.score as u32).into(),
                timestamp: r.timestamp.map_or(0, |t| t.timestamp() as u64),
            })
            .collect();

        Ok(Some(AnalysisSession {
            id: h.id.to_string(),
            user_id: h.user_id.into(),
            keyboard_id: h.keyboard_id.into(),
            corpus_id: h.corpus_id.into(),
            history,
        }))
    }
}
