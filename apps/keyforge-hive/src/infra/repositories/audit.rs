use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditRepository {
    pool: Pool<Postgres>,
}

pub struct AuditLog<'a> {
    pub action: &'a str,
    pub actor_id: Option<Uuid>,
    pub target: Option<&'a str>,
    pub details: Option<serde_json::Value>,
    pub ip: Option<String>,
    pub status_code: Option<i32>,
    pub request_id: Option<Uuid>,
    pub user_agent: Option<String>,
}

impl AuditRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn log(&self, entry: AuditLog<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs 
            (action, actor_id, target_resource, details, ip_address, status_code, request_id, user_agent)
            VALUES ($1, $2, $3, $4, $5::inet, $6, $7, $8)
            "#,
        )
        .bind(entry.action)
        .bind(entry.actor_id)
        .bind(entry.target)
        .bind(entry.details)
        .bind(entry.ip)
        .bind(entry.status_code)
        .bind(entry.request_id)
        .bind(entry.user_agent)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
