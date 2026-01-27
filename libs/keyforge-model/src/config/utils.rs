// libs/keyforge-model/src/config/utils.rs

/// Parses a "key=value" string into a tuple.
///
/// Used by clap for `value_parser`.
/// Parses a key-value pair string (e.g., "A:1.0") into a tuple.
///
/// # Errors
/// Returns an error string if the format is invalid or values cannot be parsed.
pub fn parse_key_val(s: &str) -> Result<(String, f32), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no '=' found in '{s}'"))?;
    let key = s[..pos].to_string();
    if key.is_empty() {
        return Err("invalid KEY=VALUE: empty key".to_string());
    }
    let val = s[pos + 1..]
        .parse::<f32>()
        .map_err(|e| format!("invalid value in '{s}': {e}"))?;
    Ok((key, val))
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_val() {
        assert_eq!(parse_key_val("a=1.0"), Ok(("a".to_string(), 1.0)));
        assert_eq!(parse_key_val("key=0.5"), Ok(("key".to_string(), 0.5)));

        // Error cases
        assert!(parse_key_val("invalid").is_err());
        assert!(parse_key_val("a=").is_err()); // empty value
        assert!(parse_key_val("a=not_float").is_err());
    }
}
