// apps/keyforge-hive/src/api/validation.rs

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

use crate::error::{AppError, AppResult};

use crate::constants::{MAX_FILENAME_LEN, MAX_ID_LEN, RESERVED_USERNAMES};

/// Shared helper for validating characters
fn is_valid_char(c: char, allow_dot: bool) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || (allow_dot && c == '.')
}

/// Checks if a name is a Windows reserved filename (CON, PRN, etc.)
fn is_reserved_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    reserved.contains(&stem.as_str())
        || RESERVED_USERNAMES.contains(&stem.as_str().to_lowercase().as_str())
}

/// Validates that a string is a safe filename.
/// Validates that a string is a safe filename, preventing path traversal and reserved name usage.
#[allow(dead_code)]
pub fn validate_filename(name: &str) -> AppResult<()> {
    if name.is_empty() || name.len() > MAX_FILENAME_LEN {
        return Err(AppError::Validation("Invalid filename length".into()));
    }
    if name == "." || name == ".." || is_reserved_name(name) {
        return Err(AppError::Validation("Reserved or invalid filename".into()));
    }

    for c in name.chars() {
        if !is_valid_char(c, true) {
            return Err(AppError::Validation(format!(
                "Invalid char in filename: {c}"
            )));
        }
    }
    Ok(())
}

/// Validates that a string is a safe identifier (e.g. Corpus ID).
/// Validates that a string is a safe identifier (e.g., Corpus ID), disallowing dots.
#[allow(dead_code)]
pub fn validate_id(id: &str) -> AppResult<()> {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err(AppError::Validation("Invalid ID length".into()));
    }
    if is_reserved_name(id) {
        return Err(AppError::Validation("ID cannot be a reserved name".into()));
    }

    for c in id.chars() {
        if !is_valid_char(c, false) {
            return Err(AppError::Validation(format!("Invalid char in ID: {c}")));
        }
    }
    Ok(())
}

/// Validates that a string is a safe path component.
#[allow(dead_code)]
pub fn validate_path_component(path: &str) -> AppResult<()> {
    validate_id(path)
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("valid_file.json").is_ok());
        assert!(validate_filename("").is_err());
        assert!(validate_filename("..").is_err());
        assert!(validate_filename("CON").is_err());
        assert!(validate_filename("invalid/file").is_err());
    }

    #[test]
    fn test_validate_id() {
        assert!(validate_id("valid-id_123").is_ok());
        assert!(validate_id("invalid.id").is_err());
        assert!(validate_id("AUX").is_err());
    }
}
