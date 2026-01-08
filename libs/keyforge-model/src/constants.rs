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
/// Maximum length of an author name.
pub const MAX_AUTHOR_NAME_LEN: usize = 64;
/// Maximum length of layout data string.
pub const MAX_LAYOUT_DATA_LEN: usize = 5000;

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

// --- Security Limits ---

/// Maximum size of uploaded files (bytes).
pub const MAX_INPUT_FILE_SIZE: u64 = 100 * 1024 * 1024;
/// Maximum recursion depth for JSON parsing.
pub const MAX_JSON_DEPTH: usize = 50;

// --- WebSocket Internal Signaling ---

/// Prefix for internal Job broadcast messages on the `state.tx` channel.
/// Example: "JOB:12345"
/// Note: This is NOT the external JSON protocol sent to clients.
pub const WS_MSG_JOB: &str = "JOB:";

/// Prefix for internal Cancel broadcast messages on the `state.tx` channel.
/// Example: "CANCEL:12345"
/// Note: This is NOT the external JSON protocol sent to clients.
pub const WS_MSG_CANCEL: &str = "CANCEL:";

// --- Corpus Parsing ---

/// Mapping of special token strings to their character values.
pub const CORPUS_TOKEN_MAP: &[(&str, char)] = &[
    ("SPACE", ' '),
    ("ENTER", '\n'),
    ("TAB", '\t'),
    ("BACKSPACE", '\x08'),
    ("ESCAPE", '\x1b'),
];

/// Default finger effort weights.
/// Order: Thumb, Index, Middle, Ring, Pinky.
pub const DEFAULT_FINGER_PENALTY_SCALE: &str = "1.0, 1.0, 1.1, 1.3, 1.6";

// --- Finger Effort Weights ---

/// Base effort weight for the Thumb.
pub const EFFORT_THUMB: f32 = 1.0;
/// Base effort weight for the Index finger.
pub const EFFORT_INDEX: f32 = 1.0;
/// Base effort weight for the Middle finger.
pub const EFFORT_MIDDLE: f32 = 1.1;
/// Base effort weight for the Ring finger.
pub const EFFORT_RING: f32 = 1.3;
/// Base effort weight for the Pinky finger.
pub const EFFORT_PINKY: f32 = 1.6;

// --- System Assets ---

/// Filename for the Agent configuration asset.
pub const ASSET_AGENT_CONFIG: &str = "agent";
/// Filename for the System configuration asset.
pub const ASSET_SYSTEM_CONFIG: &str = "config";
/// Filename for the Hive configuration asset.
pub const ASSET_HIVE_CONFIG: &str = "hive";
/// Filename for the Keycodes definition asset.
pub const ASSET_KEYCODES: &str = "keycodes";
/// Filename for the UI Categories asset.
pub const ASSET_UI_CATEGORIES: &str = "ui_categories";
/// Filename for the default Cost Matrix asset.
pub const ASSET_COST_MATRIX: &str = "cost_matrix";

/// Default corpus identifier.
pub const DEFAULT_CORPUS_ID: &str = "text/en_std";

// --- Corpus Injection Constants ---

/// Assumed error rate for standard prose (3%).
pub const STD_CORPUS_ERROR_RATE: f32 = 0.03;
/// Correction factor for backspace usage (1.25x error rate).
pub const STD_CORPUS_BACKSPACE_FACTOR: f32 = 1.25;
/// Average sentences per paragraph for standard prose.
pub const STD_CORPUS_SENTENCE_RATIO: f32 = 3.0;