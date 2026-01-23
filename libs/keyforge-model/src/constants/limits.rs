// libs/keyforge-model/src/constants/limits.rs

/// Maximum number of keys allowed in a keyboard definition.
pub const MAX_KEYBOARD_KEYS: usize = 200;
/// Maximum number of keys in a layout (must match keyboard).
pub const MAX_LAYOUT_KEYS: usize = MAX_KEYBOARD_KEYS;
/// Maximum number of pinned keys allowed.
pub const MAX_PINNED_KEYS_COUNT: usize = 200;
/// Maximum length of the pinned keys string representation.
pub const MAX_PINNED_KEYS_LEN: usize = 10_000;
/// Maximum length of a layout name.
pub const MAX_LAYOUT_NAME_LEN: usize = 64;
/// Minimum length of a layout name.
pub const MIN_LAYOUT_NAME_LEN: usize = 2;
/// Maximum length of a keyboard definition name.
pub const MAX_KEYBOARD_NAME_LEN: usize = 100;
/// Maximum length of a filename (e.g. cost matrix).
pub const MAX_FILENAME_LEN: usize = 255;
/// Maximum length of an author name.
pub const MAX_AUTHOR_NAME_LEN: usize = 64;
/// Maximum length of layout data string.
pub const MAX_LAYOUT_DATA_LEN: usize = 5000;
/// Minimum length of layout data string.
pub const MIN_LAYOUT_DATA_LEN: usize = 10;
/// Maximum length of a generic identifier.
pub const MAX_ID_LEN: usize = 64;
/// The total addressable space for `KeyCodes` (Unicode range).
pub const MAX_KEYCODE_SPACE: usize = 65536;

/// Maximum size of uploaded files (bytes).
pub const MAX_INPUT_FILE_SIZE: u64 = 100 * 1024 * 1024;
/// Maximum recursion depth for JSON parsing.
pub const MAX_JSON_DEPTH: usize = 50;
/// Maximum number of items in a deserialized vector (Transport Security Policy).
pub const MAX_TRANSPORT_VECTOR_ITEMS: usize = 100_000;
/// Maximum size of a session file (bytes).
pub const MAX_SESSION_FILE_SIZE: u64 = 1024 * 1024;

/// Default capacity for keyboard definition cache.
pub const DEFAULT_KB_CACHE_CAPACITY: usize = 100;
/// Default capacity for corpus cache.
pub const DEFAULT_CORPUS_CACHE_CAPACITY: usize = 50;
/// Default capacity for cost matrix cache.
pub const DEFAULT_COST_CACHE_CAPACITY: usize = 50;
/// Default capacity for keycode registry cache.
pub const DEFAULT_KEYCODE_CACHE_CAPACITY: usize = 10;
/// Default capacity for compiled engine cache.
pub const DEFAULT_ENGINE_CACHE_CAPACITY: usize = 500;

/// Minimum biometric samples required to generate a personalized profile.
pub const MIN_BIOMETRIC_SAMPLES: usize = 300;
