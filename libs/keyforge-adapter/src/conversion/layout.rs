use crate::error::{AdapterError, AdapterResult};
use keyforge_model::{KeyCode, constants::MAX_LAYOUT_DATA_LEN};
use keyforge_model::keycodes::KeycodeRegistry;

/// Strict layout-string parsing.
///
/// - Unknown tokens are treated as errors.
/// - Intended for server-side verification and other trust boundaries.
pub fn parse_layout_string_strict(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> AdapterResult<keyforge_model::Layout> {
    if s.len() > MAX_LAYOUT_DATA_LEN {
        return Err(AdapterError::LayoutTooLong(MAX_LAYOUT_DATA_LEN));
    }

    let mut keys = Vec::with_capacity(size);
    let tokens: Vec<&str> = s.split_whitespace().collect();

    for token in tokens {
        if keys.len() >= size {
            break;
        }

        // 1. Try exact match first (e.g. "MO(1)")
        if let Some(code) = registry.get_code(token) {
            keys.push(code);
            continue;
        }

        // 2. Try stripping arguments safely (e.g. "MO(1)" -> "MO")
        let base_token = if token.ends_with(')') {
            if let Some(idx) = token.find('(') {
                &token[..idx]
            } else {
                token
            }
        } else {
            token
        };

        if let Some(code) = registry.get_code(base_token) {
            keys.push(code);
        } else {
            // Strict parsing: remove length-1 ASCII backdoor. Everything must be in the registry.
            return Err(AdapterError::UnknownToken(token.to_string()));
        }
    }

    while keys.len() < size {
        keys.push(KeyCode(0));
    }

    Ok(keyforge_model::Layout::new_unchecked(keys))
}

/// Permissive layout-string parsing.
///
/// - Unknown tokens are replaced with 0 (KC_NO).
/// - Intended for UI/CLI convenience.
pub fn parse_layout_string_permissive(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> keyforge_model::Layout {
    let mut keys = Vec::with_capacity(size);
    let tokens: Vec<&str> = s.split_whitespace().collect();

    for token in tokens {
        if keys.len() >= size {
            break;
        }

        if let Some(code) = registry.get_code(token) {
            keys.push(code);
            continue;
        }

        let base_token = if token.ends_with(')') {
            if let Some(idx) = token.find('(') {
                &token[..idx]
            } else {
                token
            }
        } else {
            token
        };

        if let Some(code) = registry.get_code(base_token) {
            keys.push(code);
        } else {
            keys.push(KeyCode(0));
        }
    }

    while keys.len() < size {
        keys.push(KeyCode(0));
    }

    keyforge_model::Layout::new_unchecked(keys)
}

/// Backwards-compatible alias.
///
/// Prefer `parse_layout_string_strict` or `parse_layout_string_permissive`.
pub fn parse_layout_string(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> AdapterResult<keyforge_model::Layout> {
    parse_layout_string_strict(s, size, registry)
}
