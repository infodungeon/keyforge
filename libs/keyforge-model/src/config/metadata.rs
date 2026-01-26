// libs/keyforge-model/src/config/metadata.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Supported data types for parameters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    /// Floating point number.
    Float,
    /// Integer number.
    Integer,
    /// Boolean toggle (mapped to 0.0/1.0 in map).
    Boolean,
}

/// Metadata describing a configuration parameter.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]

pub struct ParameterMetadata {
    /// Internal key name.
    pub key: String,
    /// User-friendly label.
    pub label: String,
    /// Helpful description.
    pub description: String,
    /// Data type.
    pub param_type: ParamType,
    /// Minimum value (if numeric).
    pub min: Option<f32>,
    /// Maximum value (if numeric).
    pub max: Option<f32>,
    /// Default value.
    pub default: f32,
}
