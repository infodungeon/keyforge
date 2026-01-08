// apps/keyforge-cli/src/reports/mod.rs

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

mod benchmarks;
mod grid;
mod tables;

// pub use self::grid::print_layout as print_layout_grid;
// pub use self::tables::{scoring as print_scoring_report, statistical as print_statistical_report};

// CHANGED: scorer -> scoring
#[allow(dead_code)]
pub fn print_comparison_report(results: &[(String, keyforge_model::AnalysisReport)]) {
    let bench_data = benchmarks::load();
    tables::comparisons(results, bench_data.as_ref());
}
