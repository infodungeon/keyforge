// libs/keyforge-model/src/config/engine.rs

use crate::validator::Validator;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ts_bindings")]
use ts_rs::TS;
use utoipa::ToSchema;

/// Hardware-specific optimization parameters for the physics engines.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export))]
#[serde(default)]
pub struct EngineConfig {
    /// L1 Data Cache size in bytes.
    pub l1d_size: usize,
    /// L2 Cache size in bytes.
    pub l2_size: usize,
    /// L3 Cache size in bytes.
    pub l3_size: usize,
    /// Whether to use hardware prefetching if available.
    pub use_prefetch: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            l1d_size: 32 * 1024,
            l2_size: 256 * 1024,
            l3_size: 8 * 1024 * 1024,
            use_prefetch: true,
        }
    }
}

impl Validator for EngineConfig {
    fn validate(&self) -> Result<(), String> {
        if self.l1d_size == 0 {
            return Err("L1D size cannot be zero".into());
        }
        Ok(())
    }
}
