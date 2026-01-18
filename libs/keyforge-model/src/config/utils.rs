/// Parses a "key=value" string into a tuple.
///
/// Used by clap for `value_parser`.
/// Expects value to be an f32.
pub fn parse_key_val(s: &str) -> Result<(String, f32), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    
    let key = s[..pos].to_string();
    let val_str = &s[pos + 1..];
    
    let val = val_str
        .parse::<f32>()
        .map_err(|e| format!("invalid float value for key `{}`: {}", key, e))?;
        
    Ok((key, val))
}
