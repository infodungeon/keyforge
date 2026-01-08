// apps/keyforge-hive/src/infra/db.rs

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


use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::ConnectOptions;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Errors that can occur during database initialization and migration.
#[derive(Debug, thiserror::Error)]
pub enum DbInitError {
    /// The provided database URL is malformed.
    #[error("Invalid database URL: {0}")]
    InvalidDatabaseUrl(String),

    /// Failed to establish a connection after several retries.
    #[error("Could not connect to Postgres after {attempts} attempts: {db_url}")]
    ConnectTimeout { attempts: u32, db_url: String },

    /// Database migrations failed, possibly due to lock contention or syntax errors.
    #[error("Database migration failed after {attempts} attempts: {error}")]
    MigrationFailed { attempts: u32, error: String },
}

/// Attempts to initialize the database pool and run migrations with a retry policy.
///
/// # Errors
/// Returns [DbInitError] if the URL is invalid, connection times out, or migrations fail.
pub async fn try_init_db(db_url: &str) -> Result<PgPool, DbInitError> {
    info!("🔌 Connecting to PostgreSQL...");

    let pool = connect_with_retry(db_url).await?;

    // RETRY LOGIC for Schema Migration
    let mut attempts = 0;
    loop {
        attempts += 1;

        match sqlx::migrate!().run(&pool).await {
            Ok(_) => break,
            Err(e) => {
                if attempts >= 10 {
                    return Err(DbInitError::MigrationFailed {
                        attempts,
                        error: e.to_string(),
                    });
                }

                let is_retryable = match &e {
                    sqlx::migrate::MigrateError::Execute(inner_err) => {
                        if let Some(db_err) = inner_err.as_database_error() {
                            if let Some(code) = db_err.code() {
                                // 40P01: deadlock_detected, 23505: unique_violation
                                code == "40P01" || code == "23505"
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if is_retryable {
                    let wait = fastrand::u64(100..500);
                    warn!(
                        "⚠️  Schema lock contention (attempt {}). Retrying in {}ms...",
                        attempts, wait
                    );
                    sleep(Duration::from_millis(wait)).await;
                } else {
                    return Err(DbInitError::MigrationFailed {
                        attempts,
                        error: e.to_string(),
                    });
                }
            }
        }
    }

    info!("✅ Database connected and migrations applied.");
    Ok(pool)
}

/// Convenience wrapper used by tests and simple CLI entrypoints.
///
/// # Panics
/// Panics if database initialization or migration fails.
/// Prefer [`try_init_db`] in production application code.
pub async fn init_db(db_url: &str) -> PgPool {
    match try_init_db(db_url).await {
        Ok(p) => p,
        Err(e) => panic!("DB init failed: {}", e),
    }
}

async fn connect_with_retry(db_url: &str) -> Result<PgPool, DbInitError> {
    let max_retries = 30;
    let delay = Duration::from_secs(1);

    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let timeout_secs = env::var("DATABASE_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    info!(
        "   Pool Config: Max Conns={}, Timeout={}s",
        max_connections, timeout_secs
    );

    for i in 1..=max_retries {
        let options_res = PgConnectOptions::from_str(db_url);

        match options_res {
            Ok(mut options) => {
                options = options.log_statements(tracing::log::LevelFilter::Debug);

                match PgPoolOptions::new()
                    .max_connections(max_connections)
                    .acquire_timeout(Duration::from_secs(timeout_secs))
                    .idle_timeout(Duration::from_secs(600))
                    .max_lifetime(Duration::from_secs(1800))
                    .after_connect(|conn, _meta| Box::pin(async move {
                        use sqlx::Executor;
                        // P1 FIX: Use REPEATABLE READ to ensure FOR UPDATE SKIP LOCKED works correctly
                        conn.execute("SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL REPEATABLE READ").await?;
                        conn.execute("SET statement_timeout = '30s'").await?;
                        Ok(())
                    }))
                    .connect_with(options)
                    .await
                {
                    Ok(p) => return Ok(p),
                    Err(e) => {
                        warn!(
                            "⚠️  DB Connection attempt {}/{} failed: {}. URL: {}",
                            i, max_retries, e, db_url
                        );
                        sleep(delay).await;
                    }
                }
            }
            Err(e) => {
                return Err(DbInitError::InvalidDatabaseUrl(e.to_string()));
            }
        }
    }
    Err(DbInitError::ConnectTimeout {
        attempts: max_retries,
        db_url: db_url.to_string(),
    })
}
