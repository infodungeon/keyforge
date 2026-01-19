// apps/keyforge-hive/src/constants.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Number of characters to show when logging job IDs.
pub const LOG_JOB_ID_TRUNCATION: usize = 8;
/// Default priority for new jobs.
pub const DEFAULT_JOB_PRIORITY: i32 = 0;
/// Default status when it is unknown.
pub const DEFAULT_STATUS_UNKNOWN: &str = "unknown";
/// Default limit for listing items.
pub const DEFAULT_LIST_LIMIT: i64 = 50;
/// Confirmation key for nuke operations.
pub const NUKE_CONFIRMATION_KEY: &str = "DELETE_EVERYTHING";

/// L2 cache threshold for using table strategy (KB).
pub const TUNING_L2_CACHE_THRESHOLD: u32 = 1024;
/// OPS threshold for high-performance batch size.
pub const TUNING_OPS_THRESHOLD: f32 = 10_000_000.0;
/// Large batch size for high-performance nodes.
pub const TUNING_BATCH_SIZE_LARGE: usize = 50_000;
/// Small batch size for standard nodes.
pub const TUNING_BATCH_SIZE_SMALL: usize = 10_000;

/// Default maximum active jobs per user.
pub const DEFAULT_MAX_ACTIVE_JOBS: i32 = 5;
/// Default maximum daily jobs per user.
pub const DEFAULT_MAX_DAILY_JOBS: i32 = 50;

/// TUI Docker monitor refresh interval (seconds).
pub const TUI_DOCKER_REFRESH_INTERVAL_SECS: u64 = 2;

/// Default database URL for local development.
pub const DEFAULT_DATABASE_URL: &str =
    "postgres://keyforge:forge_password@localhost:5432/keyforge_hive";
/// Default port for the Hive server.
pub const DEFAULT_HIVE_PORT: u16 = 3000;
/// Default shutdown timeout (seconds).
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// Default Node ID for unauthenticated or unknown workers.
pub const DEFAULT_NODE_ID: &str = "unknown";
/// Default timeout for WebSocket liveness checks (seconds).
pub const WS_LIVENESS_TIMEOUT_SECS: u64 = 60;
/// Default interval for WebSocket heartbeats (seconds).
pub const WS_HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Precision for job identity hashing (Floating point normalization).
pub const JOB_IDENTITY_PRECISION: f32 = 1_000_000.0;

/// Default capacity for the event broadcast channel.
pub const DEFAULT_BROADCAST_CAPACITY: usize = 10_000;
/// Default maximum JSON body size (1 MB).
pub const DEFAULT_MAX_JSON_BODY_SIZE: usize = 1024 * 1024;

/// Default backup limit for results sample.
pub const BACKUP_RESULTS_LIMIT: i64 = 1000;

/// Reserved usernames that cannot be registered.
pub const RESERVED_USERNAMES: &[&str] =
    &["admin", "hive", "system", "root", "keyforge", "anonymous"];

/// Minimum length for a layout name.
pub const MIN_LAYOUT_NAME_LEN: usize = 2;
/// Maximum length for a layout name.
pub const MAX_LAYOUT_NAME_LEN: usize = 64;
/// Minimum length for layout data.
pub const MIN_LAYOUT_DATA_LEN: usize = 10;
/// Maximum length for layout data.
pub const MAX_LAYOUT_DATA_LEN: usize = 5000;
/// Maximum length for an author ID.
pub const MAX_AUTHOR_ID_LEN: usize = 64;
/// Maximum length for a filename.
pub const MAX_FILENAME_LEN: usize = 255;
/// Maximum length for a generic identifier.
pub const MAX_ID_LEN: usize = 64;
