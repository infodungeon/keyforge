// Copyright (c) 2025 KeyForge Contributors
//
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

/// Trait for types that require complex validation logic beyond type checking.
pub trait Validator {
    /// Validates the internal state of the object.
    /// Returns `Ok(())` if valid, or an error message if invalid.
    fn validate(&self) -> Result<(), String>;
}

/// Helper for validating layout strings.
pub struct LayoutValidator;
impl LayoutValidator {
    /// Checks if a layout string has basic structural integrity.
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
