// apps/keyforge-hive/src/infra/repositories/audit.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// Repository for managing audit logs in the database.
#[derive(Clone, Debug)]
pub struct AuditRepository {
    pool: Pool<Postgres>,
}

/// Represents a single audit log entry.
#[derive(Debug)]
pub struct AuditLog<'a> {
    /// The action performed (e.g., "`create_job`", "`delete_user`").
    pub action: &'a str,
    /// The ID of the user or system component performing the action.
    pub actor_id: Option<Uuid>,
    /// The resource affected by the action (e.g., job ID).
    pub target: Option<&'a str>,
    /// Additional context or data associated with the event.
    pub details: Option<serde_json::Value>,
    /// The IP address from which the request originated.
    pub ip: Option<String>,
    /// The HTTP status code returned by the operation.
    pub status_code: Option<i32>,
    /// The unique request ID from the tracing context.
    pub request_id: Option<Uuid>,
    /// The User-Agent string of the client.
    pub user_agent: Option<String>,
}

impl AuditRepository {
    /// Creates a new `AuditRepository` with the given database pool.
    #[must_use]
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Persists an audit log entry to the database.
    pub async fn log(&self, entry: AuditLog<'_>) -> Result<(), sqlx::Error> {
        let ip_addr: Option<sqlx::types::ipnetwork::IpNetwork> =
            entry.ip.as_deref().and_then(|s| s.parse().ok());

        sqlx::query!(
            r#"
            INSERT INTO audit_logs 
            (action, actor_id, target_resource, details, ip_address, status_code, request_id, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            entry.action,
            entry.actor_id,
            entry.target,
            entry.details,
            ip_addr,
            entry.status_code,
            entry.request_id,
            entry.user_agent
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
