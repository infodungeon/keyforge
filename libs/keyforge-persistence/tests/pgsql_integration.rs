// libs/keyforge-persistence/tests/pgsql_integration.rs

use keyforge_model::biometrics::{BiometricProfile, LatencyStats};
use keyforge_model::community::{AnalysisSession, AnalysisSessionEntry, LayoutSubmission};
use keyforge_model::layout::Layout;
use keyforge_model::types::{CorpusId, KeyboardId, LayoutId, Score, UserId};
use keyforge_model::user::{UserBiometricStatus, UserPreferences, UserProfile};
use keyforge_persistence::{
    BiometricRepository, CommunityRepository, PgUserRepository, ResearchRepository,
    SessionRepository, UserRepository,
};
use sqlx::PgPool;
use std::collections::HashMap;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use uuid::Uuid;

async fn setup_db() -> (
    PgPool,
    testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
) {
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start Postgres");
    let host = container.get_host().await.expect("Failed to get host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let pool = PgPool::connect(&url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("../../apps/keyforge-hive/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, container)
}

#[tokio::test]
async fn test_user_repository_lifecycle() {
    let (pool, _container) = setup_db().await;
    let repo = PgUserRepository::new(pool.clone());

    let user_id_uuid = Uuid::new_v4();
    let user_id = UserId::from(user_id_uuid);

    // First we need a user in the base 'users' table because of the FK constraint
    sqlx::query!(
        "INSERT INTO users (id, username) VALUES ($1, $2)",
        user_id_uuid,
        "testuser"
    )
    .execute(&pool)
    .await
    .unwrap();

    let profile = UserProfile {
        id: user_id.clone(),
        name: "Test User".to_string(),
        preferences: UserPreferences {
            space_hand: keyforge_model::types::SpaceHandPreference::Left,
            use_personal_biometrics: true,
            theme: "light".to_string(),
        },
        biometric_status: UserBiometricStatus::Ready,
    };

    // Save
    repo.save_profile(&profile)
        .await
        .expect("Failed to save profile");

    // Get
    let loaded = repo
        .get_profile(user_id_uuid)
        .await
        .expect("Failed to get profile")
        .expect("Profile not found");

    assert_eq!(loaded.name, "Test User");
    assert_eq!(loaded.preferences.theme, "light");
    assert_eq!(loaded.biometric_status, UserBiometricStatus::Ready);
}

#[tokio::test]
async fn test_biometric_repository_lifecycle() {
    let (pool, _container) = setup_db().await;
    let repo = BiometricRepository::new(pool.clone());

    let user_id_uuid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, username) VALUES ($1, $2)",
        user_id_uuid,
        "biouser"
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut bigram_latencies = HashMap::new();
    bigram_latencies.insert(
        (97, 98),
        LatencyStats {
            median_ms: 120.5,
            std_dev: 15.0,
            sample_count: 50,
        },
    );

    let profile = BiometricProfile {
        user_id: UserId::from(user_id_uuid),
        bigram_latencies,
        performance_index: 1.2,
    };

    // Save
    repo.save_profile(&profile)
        .await
        .expect("Failed to save biometric profile");

    // Get
    let loaded = repo
        .get_by_user(user_id_uuid)
        .await
        .expect("Failed to get profile")
        .expect("Profile not found");

    assert_eq!(loaded.performance_index, 1.2);
    assert_eq!(
        loaded.bigram_latencies.get(&(97, 98)).unwrap().median_ms,
        120.5
    );
}

#[tokio::test]
async fn test_community_repository_lifecycle() {
    let (pool, _container) = setup_db().await;
    let repo = CommunityRepository::new(pool.clone());

    let user_id_uuid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, username) VALUES ($1, $2)",
        user_id_uuid,
        "commuser"
    )
    .execute(&pool)
    .await
    .unwrap();

    let submission = LayoutSubmission {
        id: Uuid::new_v4().to_string(),
        author_id: UserId::from(user_id_uuid),
        keyboard_id: KeyboardId::from("corne".to_string()),
        layout: Layout::new_unchecked(vec![]),
        score: Score::from_f32(0.85).unwrap(),
        tags: vec!["ergonomic".to_string(), "split".to_string()],
        created_at: 123456789,
    };

    // Submit
    repo.submit_layout(&submission)
        .await
        .expect("Failed to submit layout");

    // Get recent
    let recent = repo
        .get_recent_submissions(10)
        .await
        .expect("Failed to get recent");
    assert!(!recent.is_empty());
    assert_eq!(recent[0].keyboard_id.as_str(), "corne");
    assert!(recent[0].tags.contains(&"ergonomic".to_string()));
}

#[tokio::test]
async fn test_session_repository_lifecycle() {
    let (pool, _container) = setup_db().await;
    let repo = SessionRepository::new(pool.clone());

    let user_id_uuid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, username) VALUES ($1, $2)",
        user_id_uuid,
        "sessionuser"
    )
    .execute(&pool)
    .await
    .unwrap();

    let session_id = Uuid::new_v4();
    let session = AnalysisSession {
        id: session_id.to_string(),
        user_id: UserId::from(user_id_uuid),
        keyboard_id: KeyboardId::from("szr35".to_string()),
        corpus_id: CorpusId::from("english".to_string()),
        history: vec![AnalysisSessionEntry {
            layout_id: LayoutId::from("rev1".to_string()),
            score: Score::from_f32(0.7).unwrap(),
            timestamp: 1600000000,
        }],
    };

    // Create
    repo.create_session(&session)
        .await
        .expect("Failed to create session");

    // Get
    let loaded = repo
        .get_session(session_id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");
    assert_eq!(loaded.keyboard_id.as_str(), "szr35");
    assert_eq!(loaded.history.len(), 1);
    assert_eq!(loaded.history[0].layout_id.as_str(), "rev1");
}

#[tokio::test]
async fn test_research_repository_lifecycle() {
    let (pool, _container) = setup_db().await;
    let repo = ResearchRepository::new(pool.clone());

    // Record
    let id = repo
        .record_metric(
            None,
            Some("test query"),
            Some("mode1"),
            Some("phase1"),
            Some(150),
            true,
            None,
            Some("engine1"),
        )
        .await
        .expect("Failed to record metric");

    assert!(id > 0);
}
