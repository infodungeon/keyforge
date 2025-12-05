// ===== keyforge/crates/keyforge-core/src/consts.rs =====
/// The full range of supported key codes (0..=65535).
/// This defines the size of the Position Map.
pub const KEY_CODE_RANGE: usize = 65536;

/// The value used to represent "No Key" or "Key Not Found" in u8 maps.
/// This effectively limits us to 254 physical keys on a keyboard.
pub const KEY_NOT_FOUND_U8: u8 = 255;

/// The number of priority tiers (Prime, Med, Low).
pub const TIER_COUNT: usize = 3;

/// Default limit for trigram evaluation in 'Fast' mode.
pub const DEFAULT_OPT_LIMIT_FAST: usize = 600;

/// Default limit for trigram evaluation in 'Slow' (High Precision) mode.
pub const DEFAULT_OPT_LIMIT_SLOW: usize = 3000;
