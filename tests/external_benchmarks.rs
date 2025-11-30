// ===== keyforge/tests/external_benchmarks.rs =====
use keyforge::api::{load_dataset, validate_layout, KeyForgeState};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::thread;

#[derive(Debug, Deserialize)]
struct CyanophageEntry {
    layout: String,
    effort: f32, // Lower is better
    sfb: f32,    // Percentage
}

fn load_cyanophage_data() -> Vec<CyanophageEntry> {
    let path = "data/benchmarks/cyanophage.json";
    let content = fs::read_to_string(path).expect("Failed to read cyanophage.json");

    match serde_json::from_str(&content) {
        Ok(data) => data,
        Err(e) => {
            panic!(
                "Failed to parse cyanophage.json: {}\nFile Content Preview:\n{}",
                e,
                content.chars().take(200).collect::<String>()
            );
        }
    }
}

fn has_real_data() -> bool {
    Path::new("data/ngrams-all.tsv").exists()
        && Path::new("data/cost_matrix.csv").exists()
        && Path::new("data/keyboards/szr35.json").exists()
        && Path::new("data/weights/ortho_split.json").exists()
        && Path::new("data/benchmarks/cyanophage.json").exists()
}

#[test]
fn test_cyanophage_ranking_correlation() {
    if !has_real_data() {
        println!("Skipping external benchmark: Real data not found");
        return;
    }

    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder
        .spawn(move || {
            let state = KeyForgeState::default();
            let _ = load_dataset(
                &state,
                "data/cost_matrix.csv",
                "data/ngrams-all.tsv",
                &Some("data/keyboards/szr35.json".to_string()),
                None,
            );

            let benchmarks = load_cyanophage_data();
            let mut kf_results = Vec::new();

            println!(
                "\n{:<20} | {:<10} | {:<10} | {:<10} | {:<10}",
                "Layout", "Cyan Rank", "KF Rank", "Cyan SFB%", "KF SFB%"
            );
            println!("{:-<70}", "-");

            // STEP 1: Collect Layout Strings
            let mut batch_jobs = Vec::new();
            {
                let guard = state.kb_def.lock().unwrap();
                let kb = guard.as_ref().unwrap();

                for b in &benchmarks {
                    // Handle "Qwerty" vs "qwerty" casing
                    let exact_key = b.layout.clone();
                    let title_key = {
                        let mut c = b.layout.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    };

                    let layout_str = if kb.layouts.contains_key(&exact_key) {
                        kb.layouts.get(&exact_key)
                    } else if kb.layouts.contains_key(&title_key) {
                        kb.layouts.get(&title_key)
                    } else {
                        None
                    };

                    if let Some(s) = layout_str {
                        batch_jobs.push((b.layout.clone(), s.clone(), b.effort, b.sfb));
                    }
                }
            }

            // STEP 2: Validate
            for (name, layout_str, cyan_effort, cyan_sfb) in batch_jobs {
                let res = validate_layout(&state, layout_str, None).expect("Validation failed");
                let kf_sfb_pct = (res.score.stat_sfb / res.score.total_bigrams) * 100.0;

                kf_results.push((
                    name,
                    res.score.layout_score,
                    kf_sfb_pct,
                    cyan_effort,
                    cyan_sfb,
                ));
            }

            if kf_results.is_empty() {
                println!("⚠️  No matching layouts found between szr35.json and cyanophage.json");
                return;
            }

            // 3. Determine Ranks
            let mut cyan_sorted = kf_results.clone();
            cyan_sorted.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());

            let mut kf_sorted = kf_results.clone();
            kf_sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            // 4. Print Comparison Table
            for item in &kf_results {
                let name = &item.0;
                let cyan_rank = cyan_sorted.iter().position(|x| x.0 == *name).unwrap() + 1;
                let kf_rank = kf_sorted.iter().position(|x| x.0 == *name).unwrap() + 1;

                let rank_diff = (cyan_rank as i32 - kf_rank as i32).abs();
                // Mark significant rank deviations (like Engram)
                let flag = if rank_diff > 5 { "(!)" } else { "" };

                println!(
                    "{:<20} | {:<10} | {:<10} | {:<10.2} | {:<10.2} {}",
                    name, cyan_rank, kf_rank, item.3, item.2, flag
                );
            }

            println!("{:-<70}", "-");

            // 5. Assertions (Strict Mode)

            // A. Sanity: QWERTY must be in the bottom tier
            if let Some(qwerty_kf_rank) = kf_sorted
                .iter()
                .position(|x| x.0.eq_ignore_ascii_case("Qwerty"))
            {
                let bottom_threshold = kf_sorted.len().saturating_sub(3);
                assert!(
                    qwerty_kf_rank >= bottom_threshold,
                    "QWERTY rank {} is too high (List size: {})",
                    qwerty_kf_rank + 1,
                    kf_sorted.len()
                );
            }

            // B. SFB Correlation
            // This will PANIC on Engram, Hands Down Ref, or Sturdy if their SFB stats diverge > 1.5%
            for item in &kf_results {
                let name = &item.0;
                let kf_sfb = item.2;
                let cy_sfb = item.4;

                // Allow small variance due to corpus differences, but fail on large physics disagreements
                if !name.eq_ignore_ascii_case("Qwerty") {
                    let diff = (kf_sfb - cy_sfb).abs();
                    assert!(
                        diff < 1.5,
                        "SFB Divergence on {}: KF={:.2}%, Cyan={:.2}%",
                        name,
                        kf_sfb,
                        cy_sfb
                    );
                }
            }
        })
        .unwrap();

    handler.join().unwrap();
}
