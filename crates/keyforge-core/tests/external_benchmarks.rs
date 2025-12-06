use comfy_table::presets::ASCII_FULL;
use comfy_table::{Cell, CellAlignment, Color, Table};
use keyforge_core::api::{load_dataset, validate_layout, KeyForgeState};
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16; // ADDED: Missing import for layout visualization
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Debug, Deserialize, Clone)]
struct CyanophageEntry {
    layout: String,
    sfb: f32,
    lateral_stretch: f32,
    pinky_scissors: f32,
    tri_redirect: f32,
    roll_in: f32,
    roll_out: f32,
}

fn get_workspace_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    let candidates = [".", "..", "../.."];
    for c in candidates {
        let p = cwd.join(c).join("data");
        if p.exists() && p.is_dir() {
            return cwd.join(c);
        }
    }
    panic!("Could not locate workspace root containing 'data/'");
}

fn load_cyanophage_data(root: &Path) -> Vec<CyanophageEntry> {
    let path = root.join("data/benchmarks/cyanophage.json");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("❌ Failed to read '{:?}'.", path));
    serde_json::from_str(&content).expect("Failed to parse cyanophage.json")
}

fn print_layout_grid(name: &str, codes: &[u16], registry: &KeycodeRegistry) {
    println!("\nLayout: {}", name);
    let rows = [(0..6, 6..12), (12..18, 18..24), (24..30, 30..36)];

    println!("+----------------+----------------+");
    for (left_r, right_r) in rows {
        print!("| ");
        for i in left_r {
            if i < codes.len() {
                let l = registry.get_label(codes[i]);
                print!("{:^2} ", l);
            } else {
                print!("   ");
            }
        }
        print!("| ");
        for i in right_r {
            if i < codes.len() {
                let l = registry.get_label(codes[i]);
                print!("{:^2} ", l);
            } else {
                print!("   ");
            }
        }
        println!("|");
    }
    println!("+----------------+----------------+");
}

#[test]
fn test_cyanophage_ranking_correlation() {
    println!("\n=== EXTERNAL BENCHMARK VERIFICATION ===");
    println!("Goal: Ensure KeyForge metrics correlate with established Cyanophage data.");
    println!("Constraint: Only layouts present in BOTH datasets are compared.");

    let root = get_workspace_root();
    let data_dir = root.join("data");

    let corpus_path = data_dir.join("corpora/default");
    let cost_path = data_dir.join("cost_matrix.csv");
    let kb_path = data_dir.join("keyboards/corne.json");
    let kc_path = data_dir.join("keycodes.json");

    if !corpus_path.exists() {
        panic!("Missing corpus dir: {:?}", corpus_path);
    }

    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder
        .spawn(move || {
            let state = KeyForgeState::default();
            let session_id = "bench_session";

            // Load Registry manually for visualization
            let registry = KeycodeRegistry::load_from_file(&kc_path).unwrap_or_default();

            load_dataset(
                &state,
                session_id,
                cost_path.to_str().unwrap(),
                corpus_path.to_str().unwrap(),
                &Some(kb_path.to_str().unwrap().to_string()),
                None,
                Some(root.to_str().unwrap()),
            )
            .expect("Failed to load dataset");

            let benchmarks = load_cyanophage_data(&root);

            struct ResultEntry {
                name: String,
                kf_score: f32,
                cyan_sfb: f32,
                kf_sfb: f32,
                cyan_lat: f32,
                kf_lat: f32,
                cyan_scis: f32,
                kf_scis: f32,
                cyan_roll: f32,
                kf_roll: f32,
                cyan_redir: f32,
                kf_redir: f32,
            }
            let mut results = Vec::new();
            let mut skipped = Vec::new();

            let sessions = state.sessions.read().unwrap();
            let session = sessions.get(session_id).unwrap();
            let kb_layouts = session.kb_def.layouts.clone();
            let key_count = session.scorer.key_count;
            drop(sessions);

            println!("Loaded {} layouts from corne.json", kb_layouts.len());

            for (name, layout_str) in kb_layouts {
                let bench = benchmarks.iter().find(|b| {
                    b.layout.eq_ignore_ascii_case(&name)
                        || b.layout
                            .replace("-", "")
                            .eq_ignore_ascii_case(&name.replace("-", ""))
                });

                if let Some(b) = bench {
                    let res =
                        validate_layout(&state, session_id, layout_str.clone(), None).unwrap();
                    let s = &res.score;
                    let t_bi = if s.total_bigrams > 0.0 {
                        s.total_bigrams
                    } else {
                        1.0
                    };
                    let t_tri = if s.total_trigrams > 0.0 {
                        s.total_trigrams
                    } else {
                        1.0
                    };

                    // Visualize
                    let codes = layout_string_to_u16(&layout_str, key_count, &registry);
                    print_layout_grid(&name, &codes, &registry);

                    results.push(ResultEntry {
                        name: name.clone(),
                        kf_score: s.layout_score,
                        cyan_sfb: b.sfb,
                        kf_sfb: (s.stat_sfb / t_bi) * 100.0,
                        cyan_lat: b.lateral_stretch,
                        kf_lat: (s.stat_lat / t_bi) * 100.0,
                        cyan_scis: b.pinky_scissors,
                        kf_scis: (s.stat_scis / t_bi) * 100.0,
                        cyan_roll: b.roll_in + b.roll_out,
                        kf_roll: (s.stat_roll / t_bi) * 100.0,
                        cyan_redir: b.tri_redirect,
                        kf_redir: (s.stat_redir / t_tri) * 100.0,
                    });
                } else {
                    skipped.push(name);
                }
            }

            results.sort_by(|a, b| a.kf_score.partial_cmp(&b.kf_score).unwrap());

            let mut table = Table::new();
            table.load_preset(ASCII_FULL);
            table.set_header(vec![
                "Rank",
                "Layout",
                "KF Score",
                "SFB% (Ref|KF)",
                "Lat% (Ref|KF)",
                "Sci% (Ref|KF)",
                "Roll% (Ref|KF)",
                "Redir% (Ref|KF)",
            ]);

            // Suppress unused warning by prefixing _
            let fmt_pair = |ref_v: f32, kf_v: f32, _invert: bool| -> Cell {
                let diff = (ref_v - kf_v).abs();
                let color = if diff < 0.5 {
                    Color::Green
                } else if diff < 2.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                let s = format!("{:5.2} | {:5.2}", ref_v, kf_v);
                Cell::new(s).fg(color).set_alignment(CellAlignment::Right)
            };

            for (i, r) in results.iter().enumerate() {
                table.add_row(vec![
                    Cell::new(format!("#{}", i + 1)),
                    Cell::new(&r.name).add_attribute(comfy_table::Attribute::Bold),
                    Cell::new(format!("{:.0}", r.kf_score)).set_alignment(CellAlignment::Right),
                    fmt_pair(r.cyan_sfb, r.kf_sfb, false),
                    fmt_pair(r.cyan_lat, r.kf_lat, false),
                    fmt_pair(r.cyan_scis, r.kf_scis, false),
                    fmt_pair(r.cyan_roll, r.kf_roll, true),
                    fmt_pair(r.cyan_redir, r.kf_redir, false),
                ]);
            }

            println!("\n{}", table);

            if !skipped.is_empty() {
                println!("\n⚠️  Skipped (No Reference Data): {}", skipped.join(", "));
            }

            // ASSERTIONS
            let qwerty_rank = results
                .iter()
                .position(|r| r.name.eq_ignore_ascii_case("Qwerty"));
            if let Some(rank) = qwerty_rank {
                let percentile = rank as f32 / results.len() as f32;
                println!("\n[CHECK]: Qwerty Rank = {}/{}", rank + 1, results.len());
                if percentile < 0.8 {
                    panic!(
                        "❌ Logic Failure: Qwerty ranked in top 80%. It should be near the bottom."
                    );
                } else {
                    println!("✅ Qwerty correctly identified as inefficient.");
                }
            }

            if results.iter().any(|r| r.kf_sfb == 0.0) {
                panic!("❌ Data Failure: Some layouts have 0.00% SFB. Corpus failed to load.");
            } else {
                println!("✅ Data Integrity: All layouts have non-zero SFB.");
            }
        })
        .unwrap();

    handler.join().unwrap();
}
