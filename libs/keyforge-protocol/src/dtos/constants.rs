// --- Validation Limits ---
pub const MAX_KEYBOARD_KEYS: usize = 200;
pub const MAX_PINNED_KEYS_COUNT: usize = 200;
pub const MAX_PINNED_KEYS_LEN: usize = 10_000;
pub const MAX_LAYOUT_NAME_LEN: usize = 64;
pub const MAX_AUTHOR_NAME_LEN: usize = 64;
pub const MAX_LAYOUT_DATA_LEN: usize = 5000;

pub const MAX_SEARCH_EPOCHS: usize = 1_000_000;
pub const MAX_SEARCH_STEPS: usize = 5_000_000;
pub const MAX_OPT_LIMIT_FAST: usize = 10_000;
pub const MAX_SAFE_WEIGHT: f32 = 100_000_000.0;
pub const MAX_LOADER_TRIGRAM_LIMIT: usize = 50_000;
pub const MAX_TEMP: f32 = 1_000.0;

// --- Physics Constants ---
pub const SCORE_SCALE: f32 = 1_000_000.0;
pub const VERIFICATION_TOLERANCE_RATIO: f32 = 0.0001;
pub const VERIFICATION_TOLERANCE_ABS_MIN: f32 = 1.0;

// --- Security Limits ---
pub const MAX_INPUT_FILE_SIZE: u64 = 100 * 1024 * 1024;
pub const MAX_JSON_DEPTH: usize = 50;

// --- WebSocket ---
pub const WS_MSG_JOB: &str = "JOB:";
pub const WS_MSG_CANCEL: &str = "CANCEL:";
