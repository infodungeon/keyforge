// -----------------------------------------------------------------------------
// File: crates/keyforge-protocol/src/constants.rs
// Purpose: Shared constants for validation and limits.
// -----------------------------------------------------------------------------

/// Maximum number of keys allowed in a keyboard geometry.
pub const MAX_KEYBOARD_KEYS: usize = 200;

/// Maximum length of the pinned_keys vector.
pub const MAX_PINNED_KEYS_COUNT: usize = 200;

/// Maximum length of the pinned_keys string (Legacy support).
pub const MAX_PINNED_KEYS_LEN: usize = 10_000;

/// Score verification: Allowable percentage difference (0.01%).
pub const VERIFICATION_TOLERANCE_RATIO: f32 = 0.0001;

/// Score verification: Minimum absolute difference allowed.
pub const VERIFICATION_TOLERANCE_ABS_MIN: f32 = 1.0;

/// Maximum length for user submission names.
pub const MAX_LAYOUT_NAME_LEN: usize = 64;

/// Maximum length for author names.
pub const MAX_AUTHOR_NAME_LEN: usize = 64;

/// Maximum length for layout string data.
pub const MAX_LAYOUT_DATA_LEN: usize = 5000;

// WebSocket Protocol Messages
pub const WS_MSG_JOB: &str = "JOB:";
pub const WS_MSG_CANCEL: &str = "CANCEL:";

// --- Validation Limits (PROTO-038) ---
pub const MAX_SEARCH_EPOCHS: usize = 1_000_000;
pub const MAX_SEARCH_STEPS: usize = 5_000_000;
pub const MAX_OPT_LIMIT_FAST: usize = 10_000;
pub const MAX_SAFE_WEIGHT: f32 = 100_000_000.0;
pub const MAX_LOADER_TRIGRAM_LIMIT: usize = 50_000;
pub const MAX_TEMP: f32 = 1_000.0;

// --- Physics Constants ---
pub const SCORE_SCALE: f32 = 1_000_000.0;

// --- Security Limits (CLI-052) ---
/// Maximum size for any user-provided input file (100MB).
pub const MAX_INPUT_FILE_SIZE: u64 = 100 * 1024 * 1024;
/// Maximum recursion depth for JSON parsing to prevent stack overflow (JSON bombs).
pub const MAX_JSON_DEPTH: usize = 50;
