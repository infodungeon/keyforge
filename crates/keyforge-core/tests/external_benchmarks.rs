use keyforge_core::api::{load_dataset, validate_layout, KeyForgeState};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

// FIX: Added Clone to derive macros
#[derive(Debug, Deserialize, Clone)]
struct CyanophageEntry {
    layout: String,
    effort: f32,
    sfb: f32,
}

fn get_workspace_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    if cwd.join("data").exists() {
        return cwd;
    }
    let up_two = cwd.join("../../");
    if up_two.join("data").exists() {
        return up_two;
    }
    panic!("Could not locate 'data' directory. CWD: {:?}", cwd);
}

fn load_cyanophage_data(root: &Path) -> Vec<CyanophageEntry> {
    let path = root.join("data/benchmarks/cyanophage.json");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("❌ Failed to read '{:?}'.", path));
    serde_json::from_str(&content).expect("Failed to parse cyanophage.json")
}

#[test]
fn test_cyanophage_ranking_correlation() {
    let root = get_workspace_root();
    let data_dir = root.join("data");

    // Check files
    let required = [
        "ngrams-all.tsv",
        "cost_matrix.csv",
        "keyboards/szr35.json",
        "benchmarks/cyanophage.json",
    ];
    for f in required {
        if !data_dir.join(f).exists() {
            panic!("Missing file: {}", f);
        }
    }

    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder
        .spawn(move || {
            let state = KeyForgeState::default();
            let session_id = "bench_session";

            // Initialize Scorer
            load_dataset(
                &state,
                session_id,
                data_dir.join("cost_matrix.csv").to_str().unwrap(),
                data_dir.join("ngrams-all.tsv").to_str().unwrap(),
                &Some(
                    data_dir
                        .join("keyboards/szr35.json")
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
                None,
                None,
            )
            .expect("Failed to load dataset");

            let benchmarks = load_cyanophage_data(&root);

            // Results Container
            struct ResultEntry {
                name: String,
                kf_rank: usize,
                cyan_rank: usize,
                cyan_sfb: f32,
                kf_sfb: f32,
                diff: f32,
            }
            let mut full_results = Vec::new();
            let mut missing_layouts = Vec::new();

            // 1. Process All Layouts
            {
                // Pre-fetch all valid layout names to avoid lock contention
                let sessions = state.sessions.lock().unwrap();
                let session = sessions.get(session_id).unwrap();
                let kb = &session.kb_def;

                // Temporary vector to hold intermediate calculations for ranking
                let mut calculated_scores = Vec::new();

                // First pass: Calculate all scores for found layouts
                for b in &benchmarks {
                    // Fuzzy match
                    let match_key = kb.layouts.keys().find(|k| {
                        k.to_lowercase() == b.layout.to_lowercase()
                            || k.to_lowercase().replace("-", "")
                                == b.layout.to_lowercase().replace("-", "")
                    });

                    if let Some(real_key) = match_key {
                        let layout_str = kb.layouts.get(real_key).unwrap().clone();
                        calculated_scores.push((b.layout.clone(), layout_str, b.sfb, b.effort));
                    } else {
                        missing_layouts.push(b.layout.clone());
                    }
                }
                drop(sessions); // Release lock

                // Second pass: Calculate scores
                let mut scored_entries = Vec::new();
                for (name, layout_str, ref_sfb, ref_effort) in calculated_scores {
                    let res = validate_layout(&state, session_id, layout_str, None).unwrap();
                    let total = if res.score.total_bigrams > 0.0 {
                        res.score.total_bigrams
                    } else {
                        1.0
                    };
                    let kf_sfb = (res.score.stat_sfb / total) * 100.0;

                    scored_entries.push((
                        name,
                        res.score.layout_score,
                        kf_sfb,
                        ref_sfb,
                        ref_effort,
                    ));
                }

                // Rank them
                let mut kf_sorted = scored_entries.clone();
                kf_sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // Sort by KF Score

                let mut cyan_sorted = benchmarks.clone();
                cyan_sorted.sort_by(|a, b| a.effort.partial_cmp(&b.effort).unwrap()); // Sort by Cyan Effort

                // Build Final Report Data
                for (name, _, kf_sfb, ref_sfb, _) in &scored_entries {
                    let kf_rank = kf_sorted.iter().position(|x| x.0 == *name).unwrap() + 1;
                    let cyan_rank = cyan_sorted.iter().position(|x| x.layout == *name).unwrap() + 1;

                    full_results.push(ResultEntry {
                        name: name.clone(),
                        kf_rank,
                        cyan_rank,
                        cyan_sfb: *ref_sfb,
                        kf_sfb: *kf_sfb,
                        diff: (*kf_sfb - *ref_sfb).abs(),
                    });
                }
            }

            // 2. Print Full Report
            println!(
                "\n{:<25} | {:<5} | {:<5} | {:<8} | {:<8} | {:<8}",
                "Layout", "Cyan#", "KF#", "Ref SFB", "KF SFB", "Diff"
            );
            println!("{:-<75}", "-");

            for res in &full_results {
                let alert = if res.diff > 1.5 { "(!)" } else { "" };
                println!(
                    "{:<25} | {:<5} | {:<5} | {:<8.2} | {:<8.2} | {:<8.2} {}",
                    res.name, res.cyan_rank, res.kf_rank, res.cyan_sfb, res.kf_sfb, res.diff, alert
                );
            }

            println!("\n⚠️  MISSING LAYOUTS (In benchmark but not in szr35.json):");
            for m in &missing_layouts {
                println!("   - {}", m);
            }

            // 3. Fail on Errors
            let mut errors = Vec::new();

            // Check Qwerty Rank
            if let Some(res) = full_results
                .iter()
                .find(|r| r.name.eq_ignore_ascii_case("Qwerty"))
            {
                let total = full_results.len();
                if res.kf_rank < total - 5 {
                    errors.push(format!("Qwerty ranked too high: {}/{}", res.kf_rank, total));
                }
            }

            // Check SFB Correlation
            for res in &full_results {
                if res.name.eq_ignore_ascii_case("Qwerty") {
                    continue;
                }
                if res.diff > 1.5 {
                    errors.push(format!(
                        "SFB Mismatch [{}]: Ref {:.2}% vs KF {:.2}% (Diff {:.2})",
                        res.name, res.cyan_sfb, res.kf_sfb, res.diff
                    ));
                }
            }

            if !errors.is_empty() {
                println!("\n❌ --- FAILURE SUMMARY ---");
                for e in &errors {
                    println!("{}", e);
                }
                panic!("Benchmark correlation failed with {} errors.", errors.len());
            }
        })
        .unwrap();

    handler.join().unwrap();
}
