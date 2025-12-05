use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub async fn init_db(db_url: &str) -> PgPool {
    info!("🔌 Connecting to PostgreSQL...");

    let pool = connect_with_retry(db_url).await;

    let schema = include_str!("../schema.sql");
    if let Err(e) = apply_schema(&pool, schema).await {
        // We log error here but panic because a DB without schema is useless.
        // However, we panic with a structured error message.
        eprintln!("❌ FATAL: Database Schema Migration Failed.");
        eprintln!("   Error: {}", e);
        panic!("Database migration failed");
    }

    info!("✅ Database connected and schema applied.");
    pool
}

async fn connect_with_retry(db_url: &str) -> PgPool {
    let max_retries = 30;
    let delay = Duration::from_secs(1);

    for i in 1..=max_retries {
        match PgPoolOptions::new()
            .max_connections(50)
            .connect(db_url)
            .await
        {
            Ok(p) => return p,
            Err(e) => {
                warn!(
                    "⚠️  DB Connection attempt {}/{} failed: {}. Retrying...",
                    i, max_retries, e
                );
                sleep(delay).await;
            }
        }
    }
    panic!("❌ FATAL: Could not connect to Postgres after 30 seconds.");
}

async fn apply_schema(pool: &PgPool, schema: &str) -> Result<(), sqlx::Error> {
    let statements = split_sql(schema);

    // Start a transaction for atomic migration
    let mut tx = pool.begin().await?;

    for (i, sql) in statements.iter().enumerate() {
        if sql.trim().is_empty() {
            continue;
        }

        if let Err(e) = sqlx::query(sql).execute(&mut *tx).await {
            if let Some(db_err) = e.as_database_error() {
                if let Some(code) = db_err.code() {
                    // Postgres codes for "duplicate/already exists"
                    // 42P07: duplicate_table
                    // 42710: duplicate_object
                    // 42723: duplicate_function
                    // 42704: duplicate_type (sometimes)
                    if ["42P07", "42710", "42723", "42704"].contains(&code.as_ref()) {
                        continue;
                    }
                }
            }

            // If it's a real error, rollback is automatic when tx is dropped/returns Err
            tracing::error!("🚨 Schema Error in statement #{}:\n{}", i + 1, sql);
            return Err(e);
        }
    }

    tx.commit().await?;
    Ok(())
}

/// Robust SQL splitter.
/// Handles:
/// 1. Inline comments (-- ...)
/// 2. Postgres Dollar Quotes ($$ ... $$) for functions
fn split_sql(raw: &str) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut current = String::new();
    let mut inside_dollar = false;

    for line in raw.lines() {
        // 1. Identify the "Code" part of the line (strip trailing comments)
        // Note: Simple check. Doesn't handle '--' inside strings, but schema is trusted source.
        let effective_line = if let Some(idx) = line.find("--") {
            &line[..idx]
        } else {
            line
        };

        let trimmed_check = effective_line.trim();

        // 2. Check for Stored Procedure delimiter ($$)
        if line.contains("$$") {
            inside_dollar = !inside_dollar;
        }

        // 3. Append the FULL line (comments and all) to the buffer
        current.push_str(line);
        current.push('\n');

        // 4. Split condition:
        // - Not inside a stored procedure
        // - The *code* part of the line ends with a semicolon
        if !inside_dollar && !trimmed_check.is_empty() && trimmed_check.ends_with(';') {
            cmds.push(current.trim().to_string());
            current = String::new();
        }
    }

    // Flush remainder
    if !current.trim().is_empty() {
        cmds.push(current.trim().to_string());
    }
    cmds
}
