use serde::{Deserialize, Deserializer};

pub fn deserialize_limited_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v: Vec<T> = Vec::deserialize(deserializer)?;
    // Hard limit of 100k items to prevent memory exhaustion
    if v.len() > 100_000 {
        return Err(serde::de::Error::custom(
            "Vector exceeds limit of 100,000 items",
        ));
    }
    Ok(v)
}
