/// Parses a "key=value" string into a tuple.
///
/// Used by clap for `value_parser`.
/// Expects value to be an f32.
/// Parses a key=value pair.
///
/// # Errors
///
/// Returns an error if the string is not in key=value format or if the value
/// is not a valid float.
pub fn parse_key_val(s: &str) -> Result<(String, f32), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    let key = s[..pos].to_string();
    let val_str = &s[pos + 1..];

    let val = val_str
        .parse::<f32>()
        .map_err(|e| format!("invalid float value for key `{key}`: {e}"))?;

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
