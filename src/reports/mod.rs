// ===== keyforge/src/reports/mod.rs =====
use comfy_table::presets::ASCII_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use keyforge::config::ScoringWeights;
use keyforge::scorer::ScoreDetails;
use serde::Deserialize;
use std::fs;
use std::path::Path;

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

fn load_benchmarks() -> Option<Vec<BenchmarkEntry>> {
    let path = "data/benchmarks/cyanophage.json";

    if !Path::new(path).exists() {
        eprintln!("⚠️  Notice: Benchmark file '{}' not found.", path);
        eprintln!("    (The 'Reality Check' table will be skipped.)");
        return None;
    }

    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("❌ Error parsing benchmark JSON: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("❌ Error reading benchmark file: {}", e);
            None
        }
    }
}

pub fn print_layout_grid(name: &str, bytes: &[u8]) {
    println!("\nLayout: {}", name);
    let mut table = Table::new();
    table.load_preset(ASCII_FULL);

    let cols = 10;

    for chunk in bytes.chunks(cols) {
        let cells: Vec<Cell> = chunk
            .iter()
            .map(|&b| {
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
        Cell::new(format!("Bas\n{:.0}", w.penalty_sfb_base)),
        Cell::new(format!("Lat\n{:.0}", w.penalty_sfb_lateral)),
        Cell::new(format!("WkL\n{:.0}", w.penalty_sfb_lateral_weak)),
        Cell::new(format!("Dia\n{:.0}", w.penalty_sfb_diagonal)),
        Cell::new(format!("Lng\n{:.0}", w.penalty_sfb_long)),
        Cell::new(format!("Bot\n{:.0}", w.penalty_sfb_bottom)),
        Cell::new("SFR"),
        Cell::new(format!("Str\n{:.0}", w.penalty_monogram_stretch)),
        Cell::new("LSB"),
        Cell::new("Lat"),
        Cell::new("Sci"),
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
    // 1. Comparison vs Best in Set (Relative)
    if !results.is_empty() {
        let best = results
            .iter()
            .min_by(|a, b| a.1.layout_score.partial_cmp(&b.1.layout_score).unwrap())
            .unwrap();
        let best_score = best.1.layout_score;

        let mut table = Table::new();
        table
            .load_preset(ASCII_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.add_row(vec![
            Cell::new(format!("Comparison vs Best ({})", best.0)).add_attribute(Attribute::Bold),
            Cell::new("Score"),
            Cell::new("Delta"),
            Cell::new("% Diff"),
        ]);

        for i in 1..=3 {
            if let Some(col) = table.column_mut(i) {
                col.set_cell_alignment(CellAlignment::Right);
            }
        }

        for (name, d) in results {
            let score = d.layout_score;
            let delta = score - best_score;
            let pct = if best_score > 0.0 {
                (delta / best_score) * 100.0
            } else {
                0.0
            };

            let name_cell = if name == &best.0 {
                Cell::new(name)
                    .fg(Color::Green)
                    .add_attribute(Attribute::Bold)
            } else {
                Cell::new(name).add_attribute(Attribute::Bold)
            };

            table.add_row(vec![
                name_cell,
                Cell::new(format!("{:.0}", score)),
                Cell::new(format!("{:.0}", delta)),
                Cell::new(format!("{:.1}%", pct)),
            ]);
        }
        println!("\n{}", table);
    }

    // 2. Reality Check vs External Benchmarks
    if let Some(benchmarks) = load_benchmarks() {
        let mut table = Table::new();
        table
            .load_preset(ASCII_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.add_row(vec![
            Cell::new("REALITY CHECK (External Data)").add_attribute(Attribute::Bold),
            Cell::new("Score").set_alignment(CellAlignment::Center),
            Cell::new("Dist").set_alignment(CellAlignment::Center),
            Cell::new("SFB%").set_alignment(CellAlignment::Center),
            Cell::new("Lat%").set_alignment(CellAlignment::Center),
            Cell::new("Sci%").set_alignment(CellAlignment::Center),
            Cell::new("Roll%").set_alignment(CellAlignment::Center),
            Cell::new("Redir%").set_alignment(CellAlignment::Center),
            Cell::new("Skip%").set_alignment(CellAlignment::Center),
        ]);

        table.add_row(vec![
            Cell::new("Layout"),
            Cell::new("Ref Effort"),
            Cell::new("Ref | KF"),
            Cell::new("Ref | KF").add_attribute(Attribute::Bold),
            Cell::new("Ref | KF"),
            Cell::new("Ref (P) | KF (T)"), // Pinky vs Total
            Cell::new("Ref | KF"),
            Cell::new("Ref | KF"),
            Cell::new("Ref | KF"),
        ]);

        for i in 1..=8 {
            if let Some(col) = table.column_mut(i) {
                col.set_cell_alignment(CellAlignment::Right);
            }
        }

        // Helper closure for color-coded cells
        let fmt_stat = |ref_val: f32, kf_val: f32| -> Cell {
            let diff = (ref_val - kf_val).abs();
            let text = format!("{:.1} | {:.1}", ref_val, kf_val);

            if diff < 0.5 {
                Cell::new(text).fg(Color::Green)
            } else if diff < 1.5 {
                Cell::new(text).fg(Color::Yellow)
            } else {
                Cell::new(text).fg(Color::Red)
            }
        };

        for (name, d) in results {
            // Fuzzy match name
            let bench = benchmarks.iter().find(|b| {
                b.layout.eq_ignore_ascii_case(name)
                    || b.layout
                        .replace(" ", "")
                        .eq_ignore_ascii_case(&name.replace(" ", ""))
            });

            if let Some(b) = bench {
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

                let kf_sfb = (d.stat_sfb / t_bi) * 100.0;
                let kf_lat = (d.stat_lat / t_bi) * 100.0;
                let kf_scis = (d.stat_scis / t_bi) * 100.0;
                let kf_roll = (d.stat_roll / t_bi) * 100.0;
                let kf_redir = (d.stat_redir / t_tri) * 100.0;
                let kf_skip = (d.stat_skip / t_tri) * 100.0;

                let ref_roll = b.roll_in + b.roll_out;

                table.add_row(vec![
                    Cell::new(name).add_attribute(Attribute::Bold),
                    Cell::new(format!("{:.2}", b.effort)).fg(Color::Yellow),
                    // Distance is not color coded (units differ)
                    Cell::new(format!("{:.0} | {:.0}", b.distance, d.geo_dist)),
                    // SFB (Priority metric)
                    fmt_stat(b.sfb, kf_sfb),
                    fmt_stat(b.lateral_stretch, kf_lat),
                    fmt_stat(b.pinky_scissors, kf_scis),
                    fmt_stat(ref_roll, kf_roll),
                    fmt_stat(b.tri_redirect, kf_redir),
                    fmt_stat(b.skip_bigrams, kf_skip),
                ]);
            }
        }
        println!("\n{}", table);
    }
}
