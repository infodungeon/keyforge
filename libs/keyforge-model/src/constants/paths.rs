// libs/keyforge-model/src/constants/paths.rs

/// Name of the configuration directory (e.g. for OS app data).
pub const CONFIG_DIR_NAME: &str = "keyforge";

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

/// Default path for user statistics.
pub const DEFAULT_USER_STATS_PATH: &str = "user_stats.jsonl";
/// Default path for personal cost profile.
pub const DEFAULT_PERSONAL_COST_PATH: &str = "personal_cost.json";

/// A list of system assets that must be present for the application to function.
pub const REQUIRED_ASSETS: &[&str] = &[
    "config/keycodes",
    "weights/cost_matrix",
    "corpora/text/en_std/1grams",
];

/// Directories containing system-provided data, models, and benchmarks.
pub const SYSTEM_DIRS: &[&str] = &[
    "system/config",
    "system/keyboards/models",
    "system/corpora/text/en_std",
    "system/weights",
    "system/benchmarks",
];

/// Directories for user-created content that should be persisted (e.g., custom layouts).
pub const USER_WORKSPACE_DIRS: &[&str] = &[
    "user/keyboards",
    "user/corpora",
    "user/weights",
    "user/config",
];

/// VOLATILE: Directories for transient data (e.g., queues, WALs).
pub const USER_RUNTIME_DIRS: &[&str] = &["user/queue", "user/agent_wal", "user/temp"];

/// Default fallback directory path.
pub const DEFAULT_FALLBACK_PATH: &str = ".";

/// Standard candidate paths for locating the data directory.
pub const DATA_DIR_CANDIDATES: &[&str] = &[
    "data",
    "../data",
    "../../data",
    "/app/data", // Docker convention
];
