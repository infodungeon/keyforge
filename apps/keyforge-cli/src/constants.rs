// apps/keyforge-cli/src/constants.rs

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

//! Centralized constants for the KeyForge CLI.

// --- Defaults ---

/// Default Hive Server URL.
pub const DEFAULT_HIVE_URL: &str = "http://localhost:3000";

/// Default configuration directory name.
pub const CONFIG_DIR_NAME: &str = "keyforge";

/// Default CLI configuration filename.
pub const CLI_CONFIG_FILENAME: &str = "cli.json";

/// Default update server URL.
pub const DEFAULT_UPDATE_URL: &str = "https://keyforge-releases.example.com/api/latest";

/// Default GitHub organization for updates.
pub const REPO_OWNER: &str = "keyforge-org";

/// Default output filename for physics debug visualization.
pub const DEFAULT_DEBUG_OUTPUT: &str = "debug_physics.svg";

// --- Limits & Thresholds ---

/// Maximum number of corpora sources allowed in CLI arguments.
pub const MAX_CLI_CORPORA: usize = 50;

/// Default number of iterations for benchmarking.
pub const DEFAULT_BENCHMARK_ITERATIONS: usize = 100_000;

/// Default row limit for listing assets.
pub const DEFAULT_LIST_LIMIT: usize = 50;

/// Default width for layout formatting.
pub const DEFAULT_FMT_WIDTH: usize = 10;

// --- Paths ---

/// Default input path for user statistics.
pub const DEFAULT_USER_STATS_PATH: &str = "data/user_stats.jsonl";

/// Default output path for generated cost profiles.
pub const DEFAULT_PERSONAL_COST_PATH: &str = "data/personal_cost.json";

/// Default benchmark data path.
pub const DEFAULT_BENCHMARK_PATH: &str = "data/benchmarks/cyanophage.json";
