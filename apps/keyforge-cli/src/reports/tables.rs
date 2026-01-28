#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/reports/tables.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You    may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use comfy_table::presets::ASCII_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use keyforge_model::AnalysisReport;

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
            Cell::new(format!("{:.0}", d.summary.score)).fg(Color::Cyan),
            Cell::new(format!("{:.2}%", d.breakdown.sfb_ratio * 100.0)).fg(Color::Red),
            Cell::new(format!("{:.0}", d.breakdown.scissors)).fg(Color::Yellow),
            Cell::new(format!("{:.0}", d.breakdown.redirects)).fg(Color::Blue),
            Cell::new(format!("{:.0}", d.breakdown.rolls)).fg(Color::Green),
            Cell::new(format!("{:.2}", d.summary.hand_balance)),
        ]);
    }
    println!("\n{table}");
}
