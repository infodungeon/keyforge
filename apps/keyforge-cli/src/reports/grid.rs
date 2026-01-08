// apps/keyforge-cli/src/reports/grid.rs

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
use comfy_table::{Cell, CellAlignment, Table};
use keyforge_model::keycodes::KeycodeRegistry;
use keyforge_model::KeyCode;

#[allow(dead_code)]
pub fn print_layout(name: &str, codes: &[KeyCode], registry: &KeycodeRegistry) {
    println!("\nLayout: {}", name);
    let mut table = Table::new();
    table.load_preset(ASCII_FULL);

    let cols = 10; // Standard visual row width for most split/ortho boards

    for chunk in codes.chunks(cols) {
        let cells: Vec<Cell> = chunk
            .iter()
            .map(|&code| {
                let label = if code == KeyCode(0) {
                    " ".to_string()
                } else {
                    registry.get_label(code)
                };
                Cell::new(label).set_alignment(CellAlignment::Center)
            })
            .collect();
        table.add_row(cells);
    }
    println!("{}", table);
}
