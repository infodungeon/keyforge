// libs/keyforge-model/src/validator.rs

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

//! Validation traits and helpers.

/// Trait for types that require complex validation logic beyond type checking.
pub trait Validator {
    /// Validates the internal state of the object.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal state is invalid.
    fn validate(&self) -> Result<(), String>;
}

/// Helper for validating layout strings.
#[derive(Debug)]
pub struct LayoutValidator;
impl LayoutValidator {
    ///
    /// # Errors
    ///
    /// Returns an error if the layout string is empty or has too few keys.
    pub fn validate_structure(layout: &str) -> Result<(), String> {
        if layout.trim().is_empty() {
            return Err("Layout is empty".to_string());
        }
        if layout.split_whitespace().count() < 10 {
            return Err("Layout has too few keys".to_string());
        }
        Ok(())
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_layout_validator() {
        assert!(LayoutValidator::validate_structure("1 2 3 4 5 6 7 8 9 10").is_ok());
        assert!(LayoutValidator::validate_structure("").is_err());
        assert!(LayoutValidator::validate_structure("1 2 3").is_err());
    }
}
