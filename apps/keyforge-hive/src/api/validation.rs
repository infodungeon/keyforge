use crate::error::{AppError, AppResult};

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
}

/// Validates that a string is a safe filename.
pub fn validate_filename(name: &str) -> AppResult<()> {
    if name.is_empty() || name.len() > 255 {
        return Err(AppError::Validation("Invalid filename length".into()));
    }
    if name == "." || name == ".." || is_reserved_name(name) {
        return Err(AppError::Validation("Reserved or invalid filename".into()));
    }

    for c in name.chars() {
        if !is_valid_char(c, true) {
            return Err(AppError::Validation(format!(
                "Invalid char in filename: {}",
                c
            )));
        }
    }
    Ok(())
}

/// Validates that a string is a safe identifier (e.g. Corpus ID).
pub fn validate_id(id: &str) -> AppResult<()> {
    if id.is_empty() || id.len() > 64 {
        return Err(AppError::Validation("Invalid ID length".into()));
    }
    if is_reserved_name(id) {
        return Err(AppError::Validation("ID cannot be a reserved name".into()));
    }

    for c in id.chars() {
        if !is_valid_char(c, false) {
            return Err(AppError::Validation(format!("Invalid char in ID: {}", c)));
        }
    }
    Ok(())
}

pub fn validate_path_component(path: &str) -> AppResult<()> {
    validate_id(path)
}
