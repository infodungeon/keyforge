// apps/keyforge-hive/src/infra/repositories/jobs/projection.rs

use keyforge_model::error::ForgeError;

/// Trait for projecting from database/external representations into domain types.
///
/// This is an anti-corruption layer pattern that allows us to transform
/// database DTOs into domain models without polluting the domain layer.
pub trait Projection<Source> {
    /// Projects from the source type into Self.
    ///
    /// # Errors
    /// Returns `ForgeError` if the projection fails due to invalid data.
    fn project(source: Source) -> Result<Self, ForgeError>
    where
        Self: Sized;
}

/// Extension trait for types that can be projected from JSON values.
pub trait JsonProjection {
    /// Projects from a JSON value into Self.
    ///
    /// # Errors
    /// Returns `ForgeError` if the JSON cannot be deserialized.
    fn project(value: serde_json::Value) -> Result<Self, ForgeError>
    where
        Self: Sized;
}

// Implement JsonProjection for domain config types that need JSON deserialization
impl JsonProjection for keyforge_model::config::ScoringWeights {
    fn project(value: serde_json::Value) -> Result<Self, ForgeError> {
        let dto: keyforge_protocol::ScoringWeightsDto =
            serde_json::from_value(value).map_err(|e| ForgeError::Serde(e.to_string()))?;
        Ok(dto.into())
    }
}

impl JsonProjection for keyforge_model::config::SearchParams {
    fn project(value: serde_json::Value) -> Result<Self, ForgeError> {
        let dto: keyforge_protocol::SearchParamsDto =
            serde_json::from_value(value).map_err(|e| ForgeError::Serde(e.to_string()))?;
        Ok(dto.into())
    }
}

// Implement JsonProjection for DTOs
impl JsonProjection for keyforge_protocol::ScoringWeightsDto {
    fn project(value: serde_json::Value) -> Result<Self, ForgeError> {
        serde_json::from_value(value).map_err(|e| ForgeError::Serde(e.to_string()))
    }
}

impl JsonProjection for keyforge_protocol::SearchParamsDto {
    fn project(value: serde_json::Value) -> Result<Self, ForgeError> {
        serde_json::from_value(value).map_err(|e| ForgeError::Serde(e.to_string()))
    }
}
