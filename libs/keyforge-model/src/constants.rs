// Copyright (c) 2025 KeyForge Contributors
//
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

// --- WebSocket ---

/// Prefix for Job messages.
pub const WS_MSG_JOB: &str = "JOB:";
/// Prefix for Cancel messages.
pub const WS_MSG_CANCEL: &str = "CANCEL:";
