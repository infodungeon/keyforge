// libs/keyforge-model/src/constants.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Global constants and limits for the KeyForge system.

// --- Validation Limits ---

/// Maximum number of keys allowed in a keyboard definition.
pub const MAX_KEYBOARD_KEYS: usize = 200;
/// Maximum number of pinned keys allowed.
pub const MAX_PINNED_KEYS_COUNT: usize = 200;
/// Maximum length of the pinned keys string representation.
pub const MAX_PINNED_KEYS_LEN: usize = 10_000;
/// Maximum length of a layout name.
pub const MAX_LAYOUT_NAME_LEN: usize = 64;
/// Maximum length of a keyboard definition name.
pub const MAX_KEYBOARD_NAME_LEN: usize = 100;
/// Maximum length of a filename (e.g. cost matrix).
pub const MAX_FILENAME_LEN: usize = 255;
/// Maximum length of an author name.
pub const MAX_AUTHOR_NAME_LEN: usize = 64;
/// Maximum length of layout data string.
pub const MAX_LAYOUT_DATA_LEN: usize = 5000;
/// Maximum length of a generic identifier.
pub const MAX_ID_LEN: usize = 64;
/// Name of the configuration directory (e.g. for OS app data).
pub const CONFIG_DIR_NAME: &str = "keyforge";
/// Minimum length of layout data string.
pub const MIN_LAYOUT_DATA_LEN: usize = 10;
/// Minimum length of a layout name.
pub const MIN_LAYOUT_NAME_LEN: usize = 2;
/// Default weight for a corpus source.
pub const DEFAULT_CORPUS_WEIGHT: f32 = 1.0;

// (Search limits moved to config::search)

// --- Physics Constants ---

/// Scaling factor for fixed-point score arithmetic.
pub const SCORE_SCALE: f32 = 1_000_000.0;
/// Tolerance ratio for score verification.
pub const VERIFICATION_TOLERANCE_RATIO: f32 = 0.0001;
/// Minimum absolute tolerance for score verification.
pub const VERIFICATION_TOLERANCE_ABS_MIN: f32 = 1.0;

/// Maximum score considered plausible for a standard layout (sanity check).
pub const MAX_PLAUSIBLE_SCORE: f32 = 10_000_000.0;
/// Maximum SFB ratio considered plausible (sanity check, 20%).
pub const MAX_PLAUSIBLE_SFB_RATIO: f32 = 0.20;

/// Number of top frequent words to consider for Arena typing practice.
pub const ARENA_TOP_WORDS_LIMIT: usize = 2000;

// --- Security Limits ---

/// Maximum size of uploaded files (bytes).
pub const MAX_INPUT_FILE_SIZE: u64 = 100 * 1024 * 1024;
/// Maximum recursion depth for JSON parsing.
pub const MAX_JSON_DEPTH: usize = 50;
/// Maximum number of items in a deserialized vector (Transport Security Policy).
pub const MAX_TRANSPORT_VECTOR_ITEMS: usize = 100_000;
/// Maximum size of a session file (bytes).
pub const MAX_SESSION_FILE_SIZE: u64 = 1024 * 1024;

// --- Corpus Parsing ---

/// Mapping of special token strings to their character values.
pub const CORPUS_TOKEN_MAP: &[(&str, char)] = &[
    ("SPACE", ' '),
    ("ENTER", '\n'),
    ("TAB", '\t'),
    ("BACKSPACE", '\x08'),
    ("ESCAPE", '\x1b'),
];

// --- System Assets ---

/// Filename for the Agent configuration asset.
pub const ASSET_AGENT_CONFIG: &str = "agent";
/// Filename for the System configuration asset.
pub const ASSET_SYSTEM_CONFIG: &str = "config";
/// Filename for the Hive configuration asset.
pub const ASSET_HIVE_CONFIG: &str = "hive";
/// Filename for the Keycodes definition asset.
pub const ASSET_KEYCODES: &str = "keycodes";
/// Filename for the Keycodes definition filename.
pub const ASSET_KEYCODES_FILENAME: &str = "keycodes.json";
/// Filename for the UI Categories asset.
pub const ASSET_UI_CATEGORIES: &str = "ui_categories";
/// Filename for the default Cost Matrix asset.
pub const ASSET_COST_MATRIX: &str = "cost_matrix";
/// Filename for the default Cost Matrix JSON file.
pub const ASSET_DEFAULT_COST_MATRIX: &str = "default_costmatrix.json";

/// Filename for monogram frequencies.
pub const ASSET_1GRAMS_FILENAME: &str = "1grams.json";
/// Filename for bigram frequencies.
pub const ASSET_2GRAMS_FILENAME: &str = "2grams.json";
/// Filename for trigram frequencies.
pub const ASSET_3GRAMS_FILENAME: &str = "3grams.json";
/// Filename for common word frequencies.
pub const ASSET_WORDS_FILENAME: &str = "words.json";

/// Default corpus identifier.
pub const DEFAULT_CORPUS_ID: &str = "text/en_std";
/// Default keyboard identifier.
pub const DEFAULT_KEYBOARD_ID: &str = "ortho_30";
/// Default URL for the Hive server.
pub const DEFAULT_HIVE_URL: &str = "https://keyforge.infodungeon.com:3000";
/// Default request timeout (seconds).
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
/// Default connection timeout (seconds).
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// Current version prefix for distributed state keys.
pub const DISTRIBUTED_KEY_VERSION: &str = "v4";
/// Time-to-live for hardware profile locks (seconds, 24 hours).
pub const PROFILE_LOCK_TTL_SECS: i64 = 86400;
/// Time-to-live for node heartbeats (seconds).
pub const HEARTBEAT_TTL_SECS: i64 = 30;

/// Default User-Agent string for the `KeyForge` client.
pub const DEFAULT_USER_AGENT: &str = "KeyForge-Client/0.9";

/// Default path for user statistics.
pub const DEFAULT_USER_STATS_PATH: &str = "data/user_stats.jsonl";
/// Default path for personal cost profile.
pub const DEFAULT_PERSONAL_COST_PATH: &str = "data/personal_cost.json";
/// Default number of iterations for benchmarks.
pub const DEFAULT_BENCHMARK_ITERATIONS: usize = 100_000;
/// Default column width for grid views.
pub const DEFAULT_GRID_WIDTH: usize = 10;

// --- Corpus Injection Constants ---

/// Assumed error rate for standard prose (3%).
pub const STD_CORPUS_ERROR_RATE: f32 = 0.03;
/// Correction factor for backspace usage (1.25x error rate).
pub const STD_CORPUS_BACKSPACE_FACTOR: f32 = 1.25;
/// Average sentences per paragraph for standard prose.
pub const STD_CORPUS_SENTENCE_RATIO: f32 = 3.0;

// --- Default Metadata ---

/// Default name for newly imported or created keyboards.
pub const DEFAULT_KEYBOARD_NAME: &str = "Untitled Board";
/// Default author name.
pub const DEFAULT_AUTHOR_NAME: &str = "Unknown";
/// Default keyboard version.
pub const DEFAULT_VERSION: &str = "1.0";
/// Default notes for KLE-imported keyboards.
pub const DEFAULT_KLE_NOTES: &str = "Imported from KLE";

// --- Cache Capacities ---

/// Default capacity for keyboard definition cache.
pub const DEFAULT_KB_CACHE_CAPACITY: usize = 100;
/// Default capacity for corpus cache.
pub const DEFAULT_CORPUS_CACHE_CAPACITY: usize = 50;
/// Default capacity for cost matrix cache.
pub const DEFAULT_COST_CACHE_CAPACITY: usize = 50;
/// Default capacity for keycode registry cache.
pub const DEFAULT_KEYCODE_CACHE_CAPACITY: usize = 10;

/// Minimum biometric samples required to generate a personalized profile.
pub const MIN_BIOMETRIC_SAMPLES: usize = 300;

/// Default capacity for compiled engine cache.
pub const DEFAULT_ENGINE_CACHE_CAPACITY: usize = 500;
/// Default TTL for compiled engine cache (30 minutes).
pub const DEFAULT_ENGINE_CACHE_TTL_SECS: u64 = 1800;

/// Prefix for assets stored in Valkey/Redis.
pub const VALKEY_ASSET_PREFIX: &str = "asset:blob";

/// Maximum number of attempts to acquire a file lock.
pub const LOCK_MAX_ATTEMPTS: usize = 10;
/// Initial delay between lock acquisition attempts (milliseconds).
pub const LOCK_INITIAL_DELAY_MS: u64 = 50;

/// Default fallback directory path.
pub const DEFAULT_FALLBACK_PATH: &str = ".";

/// Standard candidate paths for locating the data directory.
pub const DATA_DIR_CANDIDATES: &[&str] = &[
    "data",
    "../data",
    "../../data",
    "/app/data", // Docker convention
];

/// Label for No-Op keys (XXXXXXX).
pub const DEFAULT_NO_OP: &str = "XXXXXXX";
/// Label for Transparent keys (_______).
pub const DEFAULT_TRANSPARENT: &str = "_______";

/// Maximum number of violations of a single type to include in reports.
pub const MAX_REPORTED_VIOLATIONS: usize = 10;

/// Minimum temperature threshold before clipping to zero.
pub const TEMP_UNDERFLOW_THRESHOLD: f32 = 1e-10;
/// Default divisor for determining reporting interval (steps / divisor).
pub const DEFAULT_REPORT_DIVISOR: usize = 100;
/// Minimum number of steps between progress reports.
pub const MIN_REPORT_INTERVAL: usize = 1000;
/// Minimum temperature threshold for accepting worsening moves.
pub const ANNEALING_MIN_TEMP: f32 = 1e-6;

// (Defaults moved to config modules)
