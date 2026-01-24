// apps/keyforge-hive/src/infra/repositories/results.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use sqlx::{Pool, Postgres, QueryBuilder};

/// Repository for managing optimization results and population samples.
#[derive(Clone, Debug)]
pub struct ResultRepository {
    pool: Pool<Postgres>,
    max_population: usize,
}

impl ResultRepository {
    /// Creates a new `ResultRepository` with the given database pool.
    #[must_use]
    pub fn new(pool: Pool<Postgres>, max_population: usize) -> Self {
        Self {
            pool,
            max_population,
        }
    }

    /// Retrieves the top 50 layouts for a given job ID.
    #[allow(clippy::cast_possible_wrap)]
    pub async fn get_population(&self, job_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            r"
            SELECT layout 
            FROM results 
            WHERE job_id = $1 
            ORDER BY score ASC 
            LIMIT $2
            ",
            job_id,
            self.max_population as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.layout).collect())
    }

    /// Retrieves the best (lowest) score for a given job ID.
    #[allow(clippy::cast_possible_truncation)]
    pub async fn get_best_score(&self, job_id: &str) -> Result<Option<f32>, sqlx::Error> {
        let res = sqlx::query_scalar!(
            "SELECT min(score) as min_score FROM results WHERE job_id = $1",
            job_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.flatten().map(|v| v as f32))
    }

    /// Inserts a batch of results into the database.
    pub async fn insert_batch(
        &self,
        items: &[(&str, &str, f32, i64, &str)],
    ) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }

        let mut query_builder: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("INSERT INTO results (job_id, layout, score, raw_score, node_id) ");

        query_builder.push_values(items, |mut b, (job, layout, score, raw, node)| {
            b.push_bind(job)
                .push_bind(layout)
                .push_bind(f64::from(*score))
                .push_bind(*raw)
                .push_bind(node);
        });

        let query = query_builder.build();
        let mut tx = self.pool.begin().await?;
        query.execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Counts the total number of results across all jobs.
    pub async fn count_total(&self) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!("SELECT count(*) FROM results")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.unwrap_or(0))
    }

    /// Retrieves summary statistics for a given job.
    pub async fn get_stats(
        &self,
        job_id: &str,
    ) -> Result<(i64, i64, Option<f32>, Option<String>), sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT 
                count(DISTINCT node_id) as nodes,
                count(*) as samples,
                min(score) as best_score,
                (SELECT layout FROM results WHERE job_id = $1 ORDER BY score ASC LIMIT 1) as best_layout
            FROM results 
            WHERE job_id = $1
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((
            row.nodes.unwrap_or(0),
            row.samples.unwrap_or(0),
            #[allow(clippy::cast_possible_truncation)]
            row.best_score.map(|v| v as f32),
            row.best_layout,
        ))
    }

    /// Prunes results older than a certain age, keeping the top N per job.
    #[allow(clippy::cast_possible_wrap)]
    pub async fn prune_old_results(&self, days: i32, keep_top: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r"
            DELETE FROM results
            WHERE id IN (
                SELECT id FROM (
                    SELECT id,
                           ROW_NUMBER() OVER (PARTITION BY job_id ORDER BY score ASC) as rank
                    FROM results
                    WHERE created_at < NOW() - make_interval(days => $1)
                ) ranked
                WHERE rank > $2
            )
            ",
            days,
            i64::from(keep_top)
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

use crate::infra::queue::{BatchSink, PersistedRecord};
#[async_trait::async_trait]
impl BatchSink for ResultRepository {
    async fn save_batch(&self, records: Vec<PersistedRecord>) -> Result<(), String> {
        let items: Vec<(&str, &str, f32, i64, &str)> = records
            .iter()
            .map(|r| {
                (
                    r.job_id.as_str(),
                    r.layout.as_str(),
                    r.score,
                    r.raw_score,
                    r.node_id.as_str(),
                )
            })
            .collect();

        self.insert_batch(&items).await.map_err(|e| e.to_string())
    }
}
