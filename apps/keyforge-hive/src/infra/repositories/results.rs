use sqlx::{Pool, Postgres, QueryBuilder, Row};

#[derive(Clone)]
pub struct ResultRepository {
    pool: Pool<Postgres>,
}

impl ResultRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn get_population(&self, job_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT layout 
            FROM results 
            WHERE job_id = $1 
            ORDER BY score ASC 
            LIMIT 50
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| r.try_get("layout").unwrap_or_default())
            .collect())
    }

    pub async fn get_best_score(&self, job_id: &str) -> Result<Option<f32>, sqlx::Error> {
        let row = sqlx::query("SELECT min(score) as min_score FROM results WHERE job_id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            let val: Option<f64> = r.try_get("min_score")?;
            Ok(val.map(|v| v as f32))
        } else {
            Ok(None)
        }
    }

    pub async fn insert_batch(&self, items: &[(&str, &str, f32, &str)]) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO results (job_id, layout, score, node_id) ");

        query_builder.push_values(items, |mut b, (job, layout, score, node)| {
            b.push_bind(job)
                .push_bind(layout)
                .push_bind(*score as f64)
                .push_bind(node);
        });

        let query = query_builder.build();
        let mut tx = self.pool.begin().await?;
        query.execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn count_total(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM results")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    pub async fn get_stats(
        &self,
        job_id: &str,
    ) -> Result<(usize, usize, Option<f32>, Option<String>), sqlx::Error> {
        let stats = sqlx::query!(
            r#"SELECT count(DISTINCT node_id) as nodes, count(*) as samples, min(score) as best_score FROM results WHERE job_id = $1"#,
            job_id
        ).fetch_one(&self.pool).await?;

        let best_layout = if let Some(score) = stats.best_score {
            sqlx::query_scalar(
                "SELECT layout FROM results WHERE job_id = $1 AND score = $2 LIMIT 1",
            )
            .bind(job_id)
            .bind(score)
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        Ok((
            stats.nodes.unwrap_or(0) as usize,
            stats.samples.unwrap_or(0) as usize,
            stats.best_score.map(|s| s as f32),
            best_layout,
        ))
    }

    pub async fn prune_old_results(&self, days: i32, keep_top: i32) -> Result<u64, sqlx::Error> {
        let query = r#"
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
        "#;

        let result = sqlx::query(query)
            .bind(days)
            .bind(keep_top)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
