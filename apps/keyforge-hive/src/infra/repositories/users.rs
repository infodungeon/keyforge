use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pub(crate) pool: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Creates a new user. Returns None if username already exists.
    pub async fn create_user(&self, username: &str) -> Result<Option<Uuid>, sqlx::Error> {
        // SECURITY FIX: ON CONFLICT DO NOTHING prevents account takeover
        let row = sqlx::query(
            "INSERT INTO users (username) VALUES ($1) ON CONFLICT (username) DO NOTHING RETURNING id"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            Ok(Some(r.try_get("id")?))
        } else {
            Ok(None) // Username taken
        }
    }

    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        key_hash: &str,
        label: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO api_keys (user_id, key_hash, label) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(key_hash)
            .bind(label)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn validate_key(&self, key_hash: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query("SELECT 1 FROM api_keys WHERE key_hash = $1")
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await?;

        if exists.is_some() {
            let pool = self.pool.clone();
            let hash = key_hash.to_string();
            // Async touch to update last_used
            tokio::spawn(async move {
                let _ = sqlx::query(
                    "UPDATE api_keys SET last_used_at = CURRENT_TIMESTAMP WHERE key_hash = $1",
                )
                .bind(hash)
                .execute(&pool)
                .await;
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Retrieves the User ID associated with a specific API key hash.
    pub async fn get_user_by_key_hash(&self, key_hash: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query("SELECT user_id FROM api_keys WHERE key_hash = $1")
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            Ok(Some(r.try_get("user_id")?))
        } else {
            Ok(None)
        }
    }

    pub async fn check_job_quota(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT max_active_jobs, max_daily_jobs FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            let max_active: Option<i32> = r.try_get("max_active_jobs").unwrap_or(Some(5));
            let max_daily: Option<i32> = r.try_get("max_daily_jobs").unwrap_or(Some(50));

            let active_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM jobs WHERE owner_id = $1 AND status = 'active'",
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

            if active_count >= max_active.unwrap_or(5) as i64 {
                return Ok(false);
            }

            let daily_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM jobs WHERE owner_id = $1 AND created_at > NOW() - INTERVAL '24 hours'"
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

            if daily_count >= max_daily.unwrap_or(50) as i64 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
