// apps/keyforge-hive/src/infra/repositories/submissions.rs

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

use crate::features::list_submissions::SubmissionEntry;
use sqlx::{Pool, Postgres, Row};

/// Repository for managing user-submitted keyboard layouts.
#[derive(Clone, Debug)]
pub struct SubmissionRepository {
    pool: Pool<Postgres>,
}

impl SubmissionRepository {
    /// Creates a new `SubmissionRepository` with the given database pool.
    #[must_use]
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Saves a newly submitted layout to the database.
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
        Ok(i64::from(id))
    }

    /// Retrieves a list of recent submissions, up to the specified limit.
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<SubmissionEntry>, sqlx::Error> {
        let rows = sqlx::query(
            r"
            SELECT id, name, layout_str, author, submitted_at 
            FROM submissions 
            ORDER BY submitted_at DESC 
            LIMIT $1
            ",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SubmissionEntry {
                id: i64::from(r.try_get::<i32, _>("id").unwrap_or(0)),
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
