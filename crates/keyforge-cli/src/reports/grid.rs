use comfy_table::presets::ASCII_FULL;
use comfy_table::{Cell, CellAlignment, Table};
use keyforge_core::keycodes::KeycodeRegistry;

pub fn print_layout(name: &str, codes: &[u16], registry: &KeycodeRegistry) {
    println!("\nLayout: {}", name);
    let mut table = Table::new();
    table.load_preset(ASCII_FULL);

    let cols = 10; // Standard visual row width for most split/ortho boards

    for chunk in codes.chunks(cols) {
        let cells: Vec<Cell> = chunk
            .iter()
            .map(|&code| {
                let label = if code == 0 {
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
