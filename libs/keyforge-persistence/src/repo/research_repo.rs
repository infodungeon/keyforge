// libs/keyforge-persistence/src/repo/research_repo.rs

use crate::error::{PersistenceError, PersistenceResult};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for tracking research metrics and LLM usage.
#[derive(Debug, Clone)]
pub struct ResearchRepository {
    pool: PgPool,
}

impl ResearchRepository {
    /// Creates a new `ResearchRepository` with the given `PostgreSQL` pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Records a new research metric entry.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the insertion fails.
    pub async fn record_metric(
        &self,
        session_id: Option<Uuid>,
        query: Option<&str>,
        mode: Option<&str>,
        phase: Option<&str>,
        response_ms: Option<i32>,
        success: bool,
        error_message: Option<&str>,
        search_engine: Option<&str>,
    ) -> PersistenceResult<i64> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO research_metrics (
                session_id, query, mode, phase, response_ms, success, error_message, search_engine
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            session_id,
            query,
            mode,
            phase,
            response_ms,
            success,
            error_message,
            search_engine
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(id)
    }

    /// Retrieves all metrics for a specific analysis session.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the query fails.
    pub async fn get_session_metrics(
        &self,
        session_id: Uuid,
    ) -> PersistenceResult<Vec<ResearchMetricRow>> {
        let rows = sqlx::query_as!(
            ResearchMetricRow,
            r#"
            SELECT id, session_id, query, mode, phase, response_ms, success, error_message, search_engine, created_at
            FROM research_metrics
            WHERE session_id = $1
            ORDER BY created_at ASC
            "#,
            session_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(rows)
    }
}

/// Database row representation for `research_metrics`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchMetricRow {
    pub id: i64,
    pub session_id: Option<Uuid>,
    pub query: Option<String>,
    pub mode: Option<String>,
    pub phase: Option<String>,
    pub response_ms: Option<i32>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub search_engine: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
