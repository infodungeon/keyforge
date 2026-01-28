// libs/keyforge-protocol/src/user.rs

use crate::types::{SpaceHandPreferenceDto, UserIdDto};
use keyforge_model::user::UserProfile;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// DTO for `UserProfile`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct UserProfileDto {
    /// Unique user ID.
    pub id: UserIdDto,
    /// Display name.
    pub name: String,
    /// System and optimization preferences.
    pub preferences: UserPreferencesDto,
    /// Readiness of the personal biometric profile.
    pub biometric_status: String,
}

impl From<UserProfile> for UserProfileDto {
    fn from(val: UserProfile) -> Self {
        Self {
            id: val.id.into(),
            name: val.name,
            preferences: UserPreferencesDto {
                space_hand: val.preferences.space_hand.into(),
                use_personal_biometrics: val.preferences.use_personal_biometrics,
                theme: val.preferences.theme,
            },
            biometric_status: format!("{:?}", val.biometric_status).to_lowercase(),
        }
    }
}

/// DTO for `UserPreferences`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct UserPreferencesDto {
    /// Preferred spacebar hand.
    pub space_hand: SpaceHandPreferenceDto,
    /// Default usage of personal timing data.
    pub use_personal_biometrics: bool,
    /// Active UI theme.
    pub theme: String,
}

/// DTO for `BiometricProfile`.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct BiometricProfileDto {
    /// Owner of the profile.
    pub user_id: UserIdDto,
    /// Statistical latencies for character sequences (flattened).
    pub bigram_latencies: Vec<((u16, u16), LatencyStatsDto)>,
    /// Calculated typing efficiency index.
    pub performance_index: f32,
}

impl From<keyforge_model::biometrics::BiometricProfile> for BiometricProfileDto {
    fn from(val: keyforge_model::biometrics::BiometricProfile) -> Self {
        let mut latencies: Vec<_> = val
            .bigram_latencies
            .into_iter()
            .map(|(k, v)| (k, LatencyStatsDto::from(v)))
            .collect();
        latencies.sort_by_key(|(k, _)| *k);

        Self {
            user_id: val.user_id.into(),
            bigram_latencies: latencies,
            performance_index: val.performance_index,
        }
    }
}

/// DTO for `LatencyStats`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
pub struct LatencyStatsDto {
    /// Median measurement.
    pub median_ms: f32,
    /// Statistical variance.
    pub std_dev: f32,
    /// Reliability factor (sample size).
    pub sample_count: usize,
}

impl From<keyforge_model::biometrics::LatencyStats> for LatencyStatsDto {
    fn from(val: keyforge_model::biometrics::LatencyStats) -> Self {
        Self {
            median_ms: val.median_ms,
            std_dev: val.std_dev,
            sample_count: val.sample_count,
        }
    }
}
