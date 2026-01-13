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

/// Maximum number of search epochs.
pub const MAX_SEARCH_EPOCHS: usize = 1_000_000;
/// Maximum number of search steps per epoch.
pub const MAX_SEARCH_STEPS: usize = 5_000_000;
/// Maximum optimization limit for fast path.
pub const MAX_OPT_LIMIT_FAST: usize = 10_000;
/// Maximum safe weight value to prevent overflow.
pub const MAX_SAFE_WEIGHT: f32 = 100_000_000.0;
/// Maximum number of trigrams to load from corpus.
pub const MAX_LOADER_TRIGRAM_LIMIT: usize = 50_000;
/// Maximum allowed temperature for annealing.
pub const MAX_TEMP: f32 = 1_000.0;

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

/// Default User-Agent string for the KeyForge client.

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

// --- Evolution Constants ---

/// Minimum temperature threshold before clipping to zero.
pub const TEMP_UNDERFLOW_THRESHOLD: f32 = 1e-10;
/// Default divisor for determining reporting interval (steps / divisor).
pub const DEFAULT_REPORT_DIVISOR: usize = 100;
/// Minimum number of steps between progress reports.
pub const MIN_REPORT_INTERVAL: usize = 1000;
/// Default random seed for search determinism.
pub const DEFAULT_SEARCH_SEED: u64 = 42;
/// Maximum number of violations of a single type to include in reports.
pub const MAX_REPORTED_VIOLATIONS: usize = 10;
/// Minimum temperature threshold for accepting worsening moves.
pub const ANNEALING_MIN_TEMP: f32 = 1e-6;

// --- Default Values (Strings) ---

/// Default characters considered high priority (Home row candidates).
pub const DEFAULT_TIER_HIGH: &str = "etaoinshr";
/// Default characters considered medium priority.
pub const DEFAULT_TIER_MED: &str = "ldcumwfgypb.,";
/// Default characters considered low priority.
pub const DEFAULT_TIER_LOW: &str = "vkjxqz/;";

/// Default bigrams that must be optimized for.
pub const DEFAULT_CRITICAL_BIGRAMS: &str = "th,he,in,er,an,re,nd,ou";
/// Default scale factors for finger penalties (Thumb, Index, Middle, Ring, Pinky).
pub const DEFAULT_FINGER_PENALTY_SCALE: &str = "0.0,1.0,1.1,1.3,1.6";
/// Default scale factors for finger repeat penalties.
pub const DEFAULT_FINGER_REPEAT_SCALE: &str = "1.0,1.0,1.0,1.2,1.5";

/// Default scale factors for finger penalties as an array.
pub const DEFAULT_FINGER_PENALTY_SCALE_ARRAY: [f32; 5] = [0.0, 1.0, 1.1, 1.3, 1.6];
/// Default scale factors for finger repeat penalties as an array.
pub const DEFAULT_FINGER_REPEAT_SCALE_ARRAY: [f32; 5] = [1.0, 1.0, 1.0, 1.2, 1.5];
/// Default comfortable scissor pairs (Indices).
pub const DEFAULT_COMFORTABLE_SCISSORS: &str = "21,23,34";

// --- Default Values (Search) ---

/// Default number of search epochs.
pub const DEFAULT_SEARCH_EPOCHS: usize = 10_000;
/// Default number of search steps per epoch.
pub const DEFAULT_SEARCH_STEPS: usize = 100_000;
/// Default search patience.
pub const DEFAULT_SEARCH_PATIENCE: usize = 500;
/// Default search patience threshold.
pub const DEFAULT_SEARCH_PATIENCE_THRESHOLD: f32 = 0.1;
/// Default minimum temperature.
pub const DEFAULT_TEMP_MIN: f32 = 0.005;
/// Default maximum temperature.
pub const DEFAULT_TEMP_MAX: f32 = 20.0;
/// Default fast-path optimization limit.
pub const DEFAULT_OPT_LIMIT_FAST: usize = 100;
/// Default slow-path optimization limit.
pub const DEFAULT_OPT_LIMIT_SLOW: usize = 1500;
/// Default number of reheats.
pub const DEFAULT_REHEATS: usize = 3;
/// Default reheat factor.
pub const DEFAULT_REHEAT_FACTOR: f32 = 0.5;

// --- Default Values (Scoring) ---

/// Default penalty for Same Finger Repeat on a weak finger.
pub const DEFAULT_PENALTY_SFR_WEAK_FINGER: f32 = 20.0;
/// Default penalty for Same Finger Repeat involving a bad row jump.
pub const DEFAULT_PENALTY_SFR_BAD_ROW: f32 = 25.0;
/// Default penalty for lateral Same Finger Repeat.
pub const DEFAULT_PENALTY_SFR_LAT: f32 = 40.0;
/// Default penalty for lateral Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_LATERAL: f32 = 65.0;
/// Default penalty for lateral SFB on a weak finger.
pub const DEFAULT_PENALTY_SFB_LATERAL_WEAK: f32 = 160.0;
/// Default base penalty for any Same Finger Bigram.
pub const DEFAULT_PENALTY_SFB_BASE: f32 = 400.0;
/// Default additional penalty for outward rolling SFBs.
pub const DEFAULT_PENALTY_SFB_OUTWARD_ADDER: f32 = 10.0;
/// Default penalty for diagonal SFBs.
pub const DEFAULT_PENALTY_SFB_DIAGONAL: f32 = 240.0;
/// Default penalty for long-distance SFBs.
pub const DEFAULT_PENALTY_SFB_LONG: f32 = 280.0;
/// Default penalty for bottom-row SFBs.
pub const DEFAULT_PENALTY_SFB_BOTTOM: f32 = 45.0;
/// Default multiplier for SFBs on weak fingers.
pub const DEFAULT_WEIGHT_WEAK_FINGER_SFB: f32 = 2.7;

/// Default row difference threshold for "long" SFBs.
pub const DEFAULT_THRESHOLD_SFB_LONG_ROW_DIFF: i8 = 2;
/// Default row difference threshold for scissors.
pub const DEFAULT_THRESHOLD_SCISSOR_ROW_DIFF: i8 = 2;
/// Default distance threshold for reach stretches.
pub const DEFAULT_THRESHOLD_REACH_STRETCH: f32 = 1.2;

/// Default penalty for scissor (adjacent finger stretch) movements.
pub const DEFAULT_PENALTY_SCISSOR: f32 = 25.0;
/// Default penalty for ring-pinky interactions.
pub const DEFAULT_PENALTY_RING_PINKY: f32 = 1.3;
/// Default penalty for lateral movement.
pub const DEFAULT_PENALTY_LATERAL: f32 = 50.0;
/// Default penalty for single-key stretches.
pub const DEFAULT_PENALTY_MONOGRAM_STRETCH: f32 = 20.0;
/// Default penalty for skipping a key (hurdle).
pub const DEFAULT_PENALTY_SKIP: f32 = 20.0;
/// Default penalty for redirecting flow (e.g., Left -> Right -> Left).
pub const DEFAULT_PENALTY_REDIRECT: f32 = 65.0;
/// Default penalty for excessive hand alternation runs.
pub const DEFAULT_PENALTY_HAND_RUN: f32 = 5.0;
/// Default bonus (negative cost) for inward rolls.
pub const DEFAULT_BONUS_INWARD_ROLL: f32 = 40.0;
/// Default bonus for specific bigram inward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_IN: f32 = 35.0;
/// Default bonus for specific bigram outward rolls.
pub const DEFAULT_BONUS_BIGRAM_ROLL_OUT: f32 = 25.0;
/// Default penalty for high-frequency keys in medium slots.
pub const DEFAULT_PENALTY_HIGH_IN_MED: f32 = 12.0;
/// Default penalty for high-frequency keys in low slots.
pub const DEFAULT_PENALTY_HIGH_IN_LOW: f32 = 20.0;
/// Default penalty for medium-frequency keys in prime slots.
pub const DEFAULT_PENALTY_MED_IN_PRIME: f32 = 2.0;
/// Default penalty for medium-frequency keys in low slots.
pub const DEFAULT_PENALTY_MED_IN_LOW: f32 = 2.0;
/// Default penalty for low-frequency keys in prime slots.
pub const DEFAULT_PENALTY_LOW_IN_PRIME: f32 = 15.0;
/// Default penalty for low-frequency keys in medium slots.
pub const DEFAULT_PENALTY_LOW_IN_MED: f32 = 2.0;
/// Default penalty for hand imbalance.
pub const DEFAULT_PENALTY_IMBALANCE: f32 = 200.0;
/// Default maximum allowed hand imbalance ratio.
pub const DEFAULT_MAX_HAND_IMBALANCE: f32 = 0.55;
/// Default weight multiplier for vertical travel distance.
pub const DEFAULT_WEIGHT_VERTICAL_TRAVEL: f32 = 1.0;
/// Default weight multiplier for lateral travel distance.
pub const DEFAULT_WEIGHT_LATERAL_TRAVEL: f32 = 3.5;
/// Default weight multiplier for finger effort.
pub const DEFAULT_WEIGHT_FINGER_EFFORT: f32 = 2.2;

/// Default cost in milliseconds (if using time-based scoring).
pub const DEFAULT_COST_MS: f32 = 120.0;
/// Default limit on the number of trigrams to load.
pub const DEFAULT_LOADER_TRIGRAM_LIMIT: usize = 3000;
/// Default required trigram coverage.
pub const DEFAULT_TRIGRAM_COVERAGE: f32 = 0.99;
