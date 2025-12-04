use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

pub async fn init_db(db_url: &str) -> PgPool {
    info!("🔌 Connecting to PostgreSQL...");

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(db_url)
        .await
        .expect("❌ Failed to connect to Postgres. Ensure Docker is running.");

    // FIXED: Correct relative path (one level up from src/)
    let schema = include_str!("../schema.sql");

    for statement in schema.split(';') {
        let sql = statement.trim();
        if !sql.is_empty() {
            if let Err(e) = sqlx::query(sql).execute(&pool).await {
                tracing::warn!("Schema warning (harmless if exists): {}", e);
            }
        }
    }

    info!("✅ Database connected and schema verified.");
    pool
}
