// libs/keyforge-model/src/config/constraints.rs

use crate::types::{KeyIndex, KeyCode};
use crate::validator::Validator;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

/// A physical constraint forcing a specific keycode to a specific physical key index.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeyConstraint {
    /// The physical index on the keyboard.
    pub index: KeyIndex,
    /// The canonical ID of the keycode (e.g., "KC_A").
    pub key: String,
}

impl KeyConstraint {
    /// Creates a new key constraint.
    #[must_use]
    pub const fn new(index: KeyIndex, key: String) -> Self {
        Self { index, key }
    }
}

impl FromStr for KeyConstraint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format: "index:key" e.g. "0:KC_A"
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid constraint format. Expected 'index:key'".into());
        }

        let index = parts[0].parse::<u16>()
            .map_err(|_| "Invalid index in constraint".to_string())?;
        
        Ok(Self::new(KeyIndex::new(index), parts[1].to_string()))
    }
}

impl Validator for KeyConstraint {
    fn validate(&self) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Constraint key cannot be empty".into());
        }
        Ok(())
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_key_constraint_creation() {
        let c = KeyConstraint::new(KeyIndex::new(0), "KC_A".to_string());
        assert_eq!(c.index.raw(), 0);
        assert_eq!(c.key, "KC_A");
    }

    #[test]
    fn test_key_constraint_parsing() {
        let c: KeyConstraint = "10:KC_B".parse().unwrap();
        assert_eq!(c.index.raw(), 10);
        assert_eq!(c.key, "KC_B");
        
        assert!("invalid".parse::<KeyConstraint>().is_err());
        assert!("abc:KC_A".parse::<KeyConstraint>().is_err());
    }
}