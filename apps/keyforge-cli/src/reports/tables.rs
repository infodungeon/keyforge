// apps/keyforge-cli/src/reports/tables.rs

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


use super::benchmarks::BenchmarkEntry;
use comfy_table::presets::ASCII_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use keyforge_model::AnalysisReport;
use keyforge_model::config::ScoringWeights;

#[allow(dead_code)]
pub fn scoring(results: &[(String, AnalysisReport)]) {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        Cell::new("Layout").add_attribute(Attribute::Bold),
        Cell::new("Score").fg(Color::Cyan),
        Cell::new("SFB%").fg(Color::Red),
        Cell::new("Scissors").fg(Color::Yellow),
        Cell::new("Redir").fg(Color::Blue),
        Cell::new("Rolls").fg(Color::Green),
        Cell::new("Balance"),
    ]);

    for (name, d) in results {
        table.add_row(vec![
            Cell::new(name).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.0}", d.score)).fg(Color::Cyan),
            Cell::new(format!("{:.2}%", d.sfb_ratio * 100.0)).fg(Color::Red),
            Cell::new(format!("{:.0}", d.scissors)).fg(Color::Yellow),
            Cell::new(format!("{:.0}", d.redirects)).fg(Color::Blue),
            Cell::new(format!("{:.0}", d.rolls)).fg(Color::Green),
            Cell::new(format!("{:.2}", d.hand_balance)),
        ]);
    }
    println!("\n{}", table);
}

#[allow(dead_code)]
pub fn statistical(_results: &[(String, AnalysisReport)], _w: &ScoringWeights) {
    println!("(Detailed statistical report temporarily unavailable during refactor)");
}

#[allow(dead_code)]
pub fn comparisons(
    results: &[(String, AnalysisReport)],
    _benchmarks: Option<&Vec<BenchmarkEntry>>,
) {
    if !results.is_empty() {
        let best = results
            .iter()
            .min_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap())
            .unwrap();
        let best_score = best.1.score;

        let mut table = Table::new();
        table
            .load_preset(ASCII_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);
        table.add_row(vec!["Comparison vs Best", "Score", "Delta", "% Diff"]);

        for (name, d) in results {
            let score = d.score;
            let delta = score - best_score;
            let pct = if best_score > 0.0 {
                (delta / best_score) * 100.0
            } else {
                0.0
            };

            table.add_row(vec![
                Cell::new(name),
                Cell::new(format!("{:.0}", score)),
                Cell::new(format!("{:.0}", delta)),
                Cell::new(format!("{:.1}%", pct)),
            ]);
        }
        println!("\n{}", table);
    }
}
