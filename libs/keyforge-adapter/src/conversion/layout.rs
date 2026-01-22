use crate::error::{AdapterError, AdapterResult};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::{constants::MAX_LAYOUT_DATA_LEN, KeyCode};

/// Strict layout-string parsing.
///
/// - Unknown tokens are treated as errors.
/// - Intended for server-side verification and other trust boundaries.
///
/// Parses a layout string strictly.
///
/// # Errors
///
/// Returns an error if the layout string has the wrong number of keys or invalid codes.
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

        if let Some(code) = registry.resolve_token(token) {
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
/// - Unknown tokens are replaced with 0 (`KC_NO`).
/// - Intended for UI/CLI convenience.
#[must_use]
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

        if let Some(code) = registry.resolve_token(token) {
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
/// Parses a layout string.
///
/// # Errors
///
/// Returns an error if the layout string is invalid.
pub fn parse_layout_string(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> AdapterResult<keyforge_model::Layout> {
    parse_layout_string_strict(s, size, registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::keycodes::KeycodeDefinition;

    #[test]
    fn test_parse_layout_string_strict() {
        let mut registry = KeycodeRegistry::new_with_defaults();
        registry.definitions.push(KeycodeDefinition {
            code: KeyCode(10),
            id: "KC_A".into(),
            label: "A".into(),
            aliases: vec!["A".into()],
        });
        registry.rebuild_maps();

        let res = parse_layout_string_strict("A B", 2, &registry);
        assert!(res.is_err(), "Strict should fail on unknown token 'B'");

        let ok = parse_layout_string_strict("A", 2, &registry).unwrap();
        assert_eq!(ok.keys[0], KeyCode(10));
        assert_eq!(ok.keys[1], KeyCode(0)); // Padded
    }

    #[test]
    fn test_parse_layout_string_strict_extended() {
        let registry = KeycodeRegistry::new_with_defaults();

        // 1. Too long
        let long_str = "A ".repeat(MAX_LAYOUT_DATA_LEN + 1);
        assert!(parse_layout_string_strict(&long_str, 10, &registry).is_err());

        // 2. Argument stripping
        // We need a registry that knows 'MO'
        let mut reg = KeycodeRegistry::new_with_defaults();
        reg.definitions.push(KeycodeDefinition {
            code: KeyCode(100),
            id: "MO".into(),
            label: "MO".into(),
            aliases: vec![],
        });
        reg.rebuild_maps();

        let ok = parse_layout_string_strict("MO(1)", 1, &reg).unwrap();
        assert_eq!(ok.keys[0], KeyCode(100));

        // 3. malformed bracket - ends with ) but no (
        let err = parse_layout_string_strict("MO)", 1, &reg);
        assert!(err.is_err());

        // 4. Token limit
        let many = parse_layout_string_strict("MO(1) MO(2) MO(3)", 1, &reg).unwrap();
        assert_eq!(many.len(), 1);
    }

    #[test]
    fn test_parse_layout_string_permissive_extended() {
        let mut reg = KeycodeRegistry::new_with_defaults();
        reg.definitions.push(KeycodeDefinition {
            code: KeyCode(100),
            id: "MO".into(),
            label: "MO".into(),
            aliases: vec![],
        });
        reg.rebuild_maps();

        // Exact match
        let ok = parse_layout_string_permissive("MO UNKNOWN", 2, &reg);
        assert_eq!(ok.keys[0], KeyCode(100));
        assert_eq!(ok.keys[1], KeyCode(0));

        // Argument stripping
        let ok = parse_layout_string_permissive("MO(1) UNKNOWN", 2, &reg);
        assert_eq!(ok.keys[0], KeyCode(100));
        assert_eq!(ok.keys[1], KeyCode(0));

        // Malformed bracket: ends with ) but no (
        let malformed = parse_layout_string_permissive("MO)", 1, &reg);
        assert_eq!(malformed.keys[0], KeyCode(0));

        // Padding
        let padded = parse_layout_string_permissive("", 2, &reg);
        assert_eq!(padded.len(), 2);
        assert_eq!(padded.keys[0], KeyCode(0));
    }

    #[test]
    fn test_parse_layout_string_alias() {
        let registry = KeycodeRegistry::new_with_defaults();
        assert!(parse_layout_string("A", 1, &registry).is_err());
    }
}
