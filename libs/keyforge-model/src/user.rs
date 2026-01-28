// libs/keyforge-model/src/user.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! User identity and preference aggregates.

use crate::types::{SpaceHandPreference, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Central identity and configuration for a system user.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    /// Unique identifier for the user.
    pub id: UserId,
    /// Display name.
    pub name: String,
    /// User-specific system and optimization preferences.
    pub preferences: UserPreferences,
    /// Metadata regarding the user's biometric state.
    pub biometric_status: UserBiometricStatus,
}

/// User preferences for the `KeyForge` experience.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UserPreferences {
    /// Preferred hand for the spacebar.
    pub space_hand: SpaceHandPreference,
    /// Whether to use the personal biometric profile by default.
    #[serde(default)]
    pub use_personal_biometrics: bool,
    /// UI theme preference.
    #[serde(default = "default_theme")]
    pub theme: String,
}

/// High-level summary of a user's biometric data readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
pub enum UserBiometricStatus {
    /// No biometric data collected yet.
    #[default]
    Empty,
    /// Data collection in progress (insufficient for a profile).
    Collecting,
    /// Sufficient data collected to generate a profile.
    Ready,
    /// Profile generated and active.
    Active,
}

fn default_theme() -> String {
    "system".into()
}

impl UserProfile {
    /// Creates a new profile for a given user.
    #[must_use]
    pub fn new(id: UserId, name: String) -> Self {
        Self {
            id,
            name,
            preferences: UserPreferences::default(),
            biometric_status: UserBiometricStatus::Empty,
        }
    }
}
