use crate::consts::KEY_CODE_RANGE;
use serde::{Deserialize, Serialize};

/// The fundamental unit of a key on a keyboard.
/// Currently mapped to u16 (0-65535) to support QMK/ZMK ranges.
///
/// - 0..=255: Standard ASCII / ISO-8859-1
/// - 256..=65535: Layers, Macros, Combos, Special Keys
pub type KeyCode = u16;

/// A collection of KeyCodes representing a full keyboard state.
pub type Layout = Vec<KeyCode>;

/// High-performance lookup map for O(1) scoring.
/// Size is fixed to KEY_CODE_RANGE.
/// Maps KeyCode -> Physical Index on Keyboard (u8).
/// 255 (KEY_NOT_FOUND_U8) represents "Key Not Found".
pub type PosMap = Box<[u8; KEY_CODE_RANGE]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub score: f32,
    pub layout: Layout,
}
