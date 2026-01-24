// apps/keyforge-cli/src/constants.rs

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

pub use keyforge_model::constants::{
    CONFIG_DIR_NAME, DEFAULT_BENCHMARK_ITERATIONS, DEFAULT_HIVE_URL,
};

/// Maximum number of corpus sources allowed in CLI args.
pub const MAX_CLI_CORPORA: usize = 50;
/// Name of the CLI configuration file.
pub const CLI_CONFIG_FILENAME: &str = "cli.json";
/// Default width for formatted layout output.
pub const DEFAULT_FMT_WIDTH: usize = 10;
/// Default limit for list commands.
pub const DEFAULT_LIST_LIMIT: usize = 50;
/// Default path for user statistics.
pub const DEFAULT_USER_STATS_PATH: &str = "user_stats.jsonl";
/// Default path for personal cost profile.
pub const DEFAULT_PERSONAL_COST_PATH: &str = "personal_cost.json";
/// Default URL for checking updates.
pub const DEFAULT_UPDATE_URL: &str = "https://keyforge-releases.example.com/api/latest";
/// GitHub repository owner for self-updates.
pub const REPO_OWNER: &str = "your-org";
/// Default output filename for debug visualizations.
pub const DEFAULT_DEBUG_OUTPUT: &str = "debug_physics.svg";
/// Default path for benchmark reference data.
pub const DEFAULT_BENCHMARK_PATH: &str = "benchmarks/cyanophage.json";
