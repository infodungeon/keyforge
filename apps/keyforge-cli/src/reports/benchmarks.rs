// apps/keyforge-cli/src/reports/benchmarks.rs

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

use comfy_table::presets::ASCII_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use keyforge_model::AnalysisReport;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BenchmarkEntry {
    pub layout: String,
    pub effort: f32,
    pub distance: f32,
    pub sfb: f32,
    pub lateral_stretch: f32,
    pub pinky_scissors: f32,
    pub tri_redirect: f32,
    pub roll_in: f32,
    pub roll_out: f32,
    pub skip_bigrams: f32,
}

impl Default for BenchmarkEntry {
    fn default() -> Self {
        Self {
            layout: "Unknown".to_string(),
            effort: 0.0,
            distance: 0.0,
            sfb: 0.0,
            lateral_stretch: 0.0,
            pinky_scissors: 0.0,
            tri_redirect: 0.0,
            roll_in: 0.0,
            roll_out: 0.0,
            skip_bigrams: 0.0,
        }
    }
}

pub fn load(root: &std::path::Path) -> Option<Vec<BenchmarkEntry>> {
    let input = crate::constants::DEFAULT_BENCHMARK_PATH;

    let Ok(path) = crate::cli_parsers::resolve_path(input, None, root) else {
        return None;
    };

    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("❌ Error parsing benchmark JSON: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("❌ Error reading benchmark file: {e}");
            None
        }
    }
}

pub fn display(current_name: &str, report: &AnalysisReport, baselines: &[BenchmarkEntry]) {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new(current_name)
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Best Baseline").add_attribute(Attribute::Bold),
            Cell::new("Delta").add_attribute(Attribute::Bold),
        ]);

    // Simple comparison for SFB% as an example
    let current_sfb = report.sfb_ratio * 100.0;
    let min_baseline_sfb = baselines
        .iter()
        .map(|b| b.sfb)
        .fold(f32::INFINITY, f32::min);
    let sfb_delta = current_sfb - min_baseline_sfb;

    table.add_row(vec![
        Cell::new("SFB%"),
        Cell::new(format!("{current_sfb:.2}%")).fg(Color::Cyan),
        Cell::new(format!("{min_baseline_sfb:.2}%")),
        Cell::new(format!("{sfb_delta:+.2}%")).fg(if sfb_delta <= 0.0 {
            Color::Green
        } else {
            Color::Red
        }),
    ]);

    // Distance
    let current_dist = report.travel_per_key;
    let min_baseline_dist = baselines
        .iter()
        .map(|b| b.distance)
        .fold(f32::INFINITY, f32::min);
    let dist_delta = current_dist - min_baseline_dist;

    table.add_row(vec![
        Cell::new("Distance/Key"),
        Cell::new(format!("{current_dist:.3}")).fg(Color::Cyan),
        Cell::new(format!("{min_baseline_dist:.3}")),
        Cell::new(format!("{dist_delta:+.3}")).fg(if dist_delta <= 0.0 {
            Color::Green
        } else {
            Color::Red
        }),
    ]);

    // Scissors
    let current_scissors = report.scissors;
    let min_baseline_scissors = baselines
        .iter()
        .map(|b| b.pinky_scissors)
        .fold(f32::INFINITY, f32::min);
    let scissors_delta = current_scissors - min_baseline_scissors;

    table.add_row(vec![
        Cell::new("Scissors"),
        Cell::new(format!("{current_scissors:.3}")).fg(Color::Cyan),
        Cell::new(format!("{min_baseline_scissors:.3}")),
        Cell::new(format!("{scissors_delta:+.3}")).fg(if scissors_delta <= 0.0 {
            Color::Green
        } else {
            Color::Red
        }),
    ]);

    // Redirects
    let current_redirs = report.redirects;
    let min_baseline_redirs = baselines
        .iter()
        .map(|b| b.tri_redirect)
        .fold(f32::INFINITY, f32::min);
    let redirs_delta = current_redirs - min_baseline_redirs;

    table.add_row(vec![
        Cell::new("Redirects"),
        Cell::new(format!("{current_redirs:.3}")).fg(Color::Cyan),
        Cell::new(format!("{min_baseline_redirs:.3}")),
        Cell::new(format!("{redirs_delta:+.3}")).fg(if redirs_delta <= 0.0 {
            Color::Green
        } else {
            Color::Red
        }),
    ]);

    println!("\n📊 Reality Check (Baseline Comparison)");
    println!("{table}");
}
