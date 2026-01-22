// libs/keyforge-model/src/constants/mod.rs

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

pub mod limits;
pub mod paths;
pub mod physics;

// Re-export for backward compatibility
pub use limits::*;
pub use paths::*;
pub use physics::*;

// --- General Defaults ---

/// Default name for newly imported or created keyboards.
pub const DEFAULT_KEYBOARD_NAME: &str = "Untitled Board";
/// Default author name.
pub const DEFAULT_AUTHOR_NAME: &str = "Unknown";
/// Default keyboard version.
pub const DEFAULT_VERSION: &str = "1.0";
/// Default notes for KLE-imported keyboards.
pub const DEFAULT_KLE_NOTES: &str = "Imported from KLE";

/// Default URL for the Hive server.
pub const DEFAULT_HIVE_URL: &str = "https://keyforge.infodungeon.com:3000";
/// Default URL for the Asset server.
pub const DEFAULT_ASSET_URL: &str = "http://localhost:3001";
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

/// Default number of iterations for benchmarks.
pub const DEFAULT_BENCHMARK_ITERATIONS: usize = 100_000;

/// Prefix for assets stored in Valkey/Redis.
pub const VALKEY_ASSET_PREFIX: &str = "asset:blob";

/// Maximum number of attempts to acquire a file lock.
pub const LOCK_MAX_ATTEMPTS: usize = 10;
/// Initial delay between lock acquisition attempts (milliseconds).
pub const LOCK_INITIAL_DELAY_MS: u64 = 50;

/// Default width for layout grid display.
pub const DEFAULT_GRID_WIDTH: usize = 10;

/// Label for No-Op keys (XXXXXXX).
pub const DEFAULT_NO_OP: &str = "XXXXXXX";
/// Label for Transparent keys (_______).
pub const DEFAULT_TRANSPARENT: &str = "_______";

/// Maximum number of swap suggestions to return.
pub const MAX_SWAP_SUGGESTIONS: usize = 5;

/// Minimum percentage improvement required to suggest a swap.
pub const MIN_SUGGESTION_IMPROVEMENT_PCT: f32 = 0.01;

/// Maximum number of violations of a single type to include in reports.
pub const MAX_REPORTED_VIOLATIONS: usize = 10;

/// Mapping of special token strings to their character values.
pub const CORPUS_TOKEN_MAP: &[(&str, char)] = &[
    ("SPACE", ' '),
    ("ENTER", '\n'),
    ("TAB", '\t'),
    ("BACKSPACE", '\x08'),
    ("ESCAPE", '\x1b'),
];
