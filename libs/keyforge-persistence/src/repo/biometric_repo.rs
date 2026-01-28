// libs/keyforge-persistence/src/repo/biometric_repo.rs

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use crate::error::{PersistenceError, PersistenceResult};
use keyforge_model::biometrics::BiometricProfile;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing biometric typing profiles.
#[derive(Debug, Clone)]
pub struct BiometricRepository {
    pool: PgPool,
}

impl BiometricRepository {
    /// Creates a new `BiometricRepository`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Saves a full biometric profile including sparse bigram latencies.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if any database operation fails.
    pub async fn save_profile(&self, profile: &BiometricProfile) -> PersistenceResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // 1. Upsert profile header
        let profile_id = sqlx::query_scalar!(
            r#"
            INSERT INTO biometric_profiles (user_id, performance_index)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE SET
                performance_index = EXCLUDED.performance_index
            RETURNING id
            "#,
            profile.user_id.as_uuid(),
            profile.performance_index
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // 2. Clear old latencies
        sqlx::query!(
            "DELETE FROM biometric_latencies WHERE profile_id = $1",
            profile_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // 3. Batch insert new latencies
        for ((k1, k2), stats) in &profile.bigram_latencies {
            sqlx::query!(
                r#"
                INSERT INTO biometric_latencies (profile_id, key1_code, key2_code, median_ms, std_dev, sample_count)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                profile_id,
                i32::from(*k1),
                i32::from(*k2),
                stats.median_ms,
                stats.std_dev,
                stats.sample_count as i32
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    /// Retrieves a biometric profile for a user.
    ///
    /// # Errors
    /// Returns `PersistenceError::Database` if the query fails.
    pub async fn get_by_user(&self, user_id: Uuid) -> PersistenceResult<Option<BiometricProfile>> {
        let header = sqlx::query!(
            "SELECT id, performance_index FROM biometric_profiles WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        let Some(h) = header else {
            return Ok(None);
        };

        let latencies = sqlx::query!(
            "SELECT key1_code, key2_code, median_ms, std_dev, sample_count FROM biometric_latencies WHERE profile_id = $1",
            h.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        let mut bigram_latencies = std::collections::HashMap::new();
        for l in latencies {
            bigram_latencies.insert(
                (l.key1_code as u16, l.key2_code as u16),
                keyforge_model::biometrics::LatencyStats {
                    median_ms: l.median_ms,
                    std_dev: l.std_dev,
                    sample_count: l.sample_count as usize,
                },
            );
        }

        Ok(Some(BiometricProfile {
            user_id: user_id.into(),
            bigram_latencies,
            performance_index: h.performance_index,
        }))
    }
}
