use comfy_table::presets::ASCII_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use keyforge::config::ScoringWeights;
use keyforge::scorer::ScoreDetails;

// Helper to convert layout name/str into displayable format is removed here
// as we rely on passing byte slices now.

pub fn print_layout_grid(name: &str, bytes: &[u8]) {
    println!("\nLayout: {}", name);
    let mut table = Table::new();
    table.load_preset(ASCII_FULL);

    // Simple visual grid logic: try to break into rows of 10
    // If it's a 30 key layout, 10x3. If 42, 10-ish?
    // Ideally this adapts to the geometry rows, but we only have raw bytes here.
    // Defaulting to 10 columns for visualization.
    let cols = 10;

    for chunk in bytes.chunks(cols) {
        let cells: Vec<Cell> = chunk
            .iter()
            .map(|&b| {
                // If 0, print empty, else print char
                let s = if b == 0 {
                    " ".to_string()
                } else {
                    (b as char).to_string()
                };
                Cell::new(s).set_alignment(CellAlignment::Center)
            })
            .collect();
        table.add_row(cells);
    }
    println!("{}", table);
}

pub fn print_scoring_report(results: &[(String, ScoreDetails)]) {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        Cell::new("Layout").add_attribute(Attribute::Bold),
        Cell::new("Total").fg(Color::Cyan),
        Cell::new("Travel"),
        Cell::new("Fing"),
        Cell::new("Strch"),
        Cell::new("Tier"),
        Cell::new("Imbal"),
        Cell::new("SFR").fg(Color::Red),
        Cell::new("SFB").fg(Color::Red),
        Cell::new("Lat"),
        Cell::new("Scis"),
        Cell::new("Run"),
        Cell::new("Redir"),
        Cell::new("Skip"),
        Cell::new("Roll").fg(Color::Green),
        Cell::new("Net").add_attribute(Attribute::Bold),
    ]);

    for i in 1..=15 {
        if let Some(col) = table.column_mut(i) {
            col.set_cell_alignment(CellAlignment::Right);
        }
    }

    for (name, d) in results {
        let roll_bonus_total = d.flow_roll_in + d.flow_roll_out + d.flow_roll_tri;
        let implied_run_cost = d.flow_cost - d.flow_redirect - d.flow_skip + roll_bonus_total;
        let total_sfb_cost = d.mech_sfb
            + d.mech_sfb_lat
            + d.mech_sfb_lat_weak
            + d.mech_sfb_diag
            + d.mech_sfb_long
            + d.mech_sfb_bot;

        table.add_row(vec![
            Cell::new(name).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.0}", d.layout_score)).fg(Color::Cyan),
            Cell::new(format!("{:.0}", d.geo_dist)),
            Cell::new(format!("{:.0}", d.finger_use)),
            Cell::new(format!("{:.0}", d.mech_mono_stretch)),
            Cell::new(format!("{:.0}", d.tier_penalty)),
            Cell::new(format!("{:.0}", d.imbalance_penalty)),
            Cell::new(format!("{:.0}", d.mech_sfr)).fg(Color::Red),
            Cell::new(format!("{:.0}", total_sfb_cost)).fg(Color::Red),
            Cell::new(format!("{:.0}", d.mech_lat)),
            Cell::new(format!("{:.0}", d.mech_scis)),
            Cell::new(format!("{:.0}", implied_run_cost)),
            Cell::new(format!("{:.0}", d.flow_redirect)),
            Cell::new(format!("{:.0}", d.flow_skip)),
            Cell::new(format!("{:.0}", roll_bonus_total)).fg(Color::Green),
            Cell::new(format!("{:.0}", d.flow_cost)).add_attribute(Attribute::Bold),
        ]);
    }
    println!("\n{}", table);
}

pub fn print_statistical_report(results: &[(String, ScoreDetails)], w: &ScoringWeights) {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        Cell::new("Layout").add_attribute(Attribute::Bold),
        // SFB
        Cell::new(format!("Bas\n{:.0}", w.penalty_sfb_base)),
        Cell::new(format!("Lat\n{:.0}", w.penalty_sfb_lateral)),
        Cell::new(format!("WkL\n{:.0}", w.penalty_sfb_lateral_weak)),
        Cell::new(format!("Dia\n{:.0}", w.penalty_sfb_diagonal)),
        Cell::new(format!("Lng\n{:.0}", w.penalty_sfb_long)),
        Cell::new(format!("Bot\n{:.0}", w.penalty_sfb_bottom)),
        // Mechanics
        Cell::new("SFR"),
        Cell::new(format!("Str\n{:.0}", w.penalty_monogram_stretch)),
        Cell::new("LSB"),
        Cell::new("Lat"),
        Cell::new("Sci"),
        // Flow
        Cell::new(format!("Red\n{:.0}", w.penalty_redirect)),
        Cell::new(format!("Skp\n{:.0}", w.penalty_skip)),
        Cell::new(format!("R2I\n{:.0}", w.bonus_bigram_roll_in)),
        Cell::new(format!("R2O\n{:.0}", w.bonus_bigram_roll_out)),
        Cell::new(format!("R3I\n{:.0}", w.bonus_inward_roll)),
        Cell::new("R3O"),
        Cell::new("Pnk"),
    ]);

    for i in 1..=18 {
        if let Some(col) = table.column_mut(i) {
            col.set_cell_alignment(CellAlignment::Right);
        }
    }

    for (name, d) in results {
        let t_bi = if d.total_bigrams > 0.0 {
            d.total_bigrams
        } else {
            1.0
        };
        let t_tri = if d.total_trigrams > 0.0 {
            d.total_trigrams
        } else {
            1.0
        };
        let t_char = if d.total_chars > 0.0 {
            d.total_chars
        } else {
            1.0
        };

        table.add_row(vec![
            Cell::new(name).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}", (d.stat_sfb_base / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_sfb_lat / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_sfb_lat_weak / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_sfb_diag / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_sfb_long / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_sfb_bot / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_sfr / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_mono_stretch / t_char) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_lsb / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_lat / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_scis / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_redir / t_tri) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_skip / t_tri) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_roll_in / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_roll_out / t_bi) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_roll3_in / t_tri) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_roll3_out / t_tri) * 100.0)),
            Cell::new(format!("{:.2}", (d.stat_pinky_reach / t_char) * 100.0)),
        ]);
    }
    println!("\n{}", table);
}

pub fn print_comparison_report(results: &[(String, ScoreDetails)]) {
    // We remove the hardcoded Reference stats here as requested by the architectural critique.
    // In a future step, this should dynamically load reference layouts and score them.
    // For now, we print a simple summary table to avoid compile errors.

    let mut table = Table::new();
    table.load_preset(ASCII_FULL);

    table.add_row(vec![
        Cell::new("Layout Comparison").add_attribute(Attribute::Bold),
        Cell::new("Score"),
        Cell::new("Diff from Best"),
    ]);

    let best_score = if !results.is_empty() {
        results[0].1.layout_score
    } else {
        0.0
    };

    for (name, d) in results {
        let diff = d.layout_score - best_score;
        table.add_row(vec![
            Cell::new(name),
            Cell::new(format!("{:.2}", d.layout_score)),
            Cell::new(format!("{:.2}", diff)),
        ]);
    }
    println!("\n{}", table);
}
