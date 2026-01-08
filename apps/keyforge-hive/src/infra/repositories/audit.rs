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
