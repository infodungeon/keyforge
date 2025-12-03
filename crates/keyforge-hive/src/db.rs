use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite};
use std::str::FromStr;
use tracing::info;

pub async fn init_db(db_url: &str) -> Pool<Sqlite> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        info!("Creating database: {}", db_url);
        Sqlite::create_database(db_url).await.unwrap();
    }

    // Configure connection options to enable WAL mode
    let options = SqliteConnectOptions::from_str(db_url)
        .expect("Invalid DB URL")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .expect("Failed to connect to database");

    // Initialize Schema
    // ADDED: The 'submissions' table definition here
    let schema = r#"
    CREATE TABLE IF NOT EXISTS jobs (
        id TEXT PRIMARY KEY,
        geometry_json TEXT NOT NULL,
        weights_json TEXT NOT NULL,
        pinned_keys TEXT NOT NULL,
        corpus_name TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS results (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        job_id TEXT NOT NULL,
        layout TEXT NOT NULL,
        score REAL NOT NULL,
        node_id TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(job_id) REFERENCES jobs(id)
    );
    CREATE INDEX IF NOT EXISTS idx_results_job_score ON results(job_id, score);

    CREATE TABLE IF NOT EXISTS submissions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        layout_str TEXT NOT NULL,
        author TEXT DEFAULT 'Anonymous',
        notes TEXT,
        status TEXT DEFAULT 'pending',
        submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    "#;

    sqlx::query(schema)
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    info!("💾 Database connected and migrated (WAL Mode).");
    pool
}
