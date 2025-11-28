use std::path::Path;
use std::process::Command;

fn has_real_data() -> bool {
    Path::new("data/ngrams-all.tsv").exists() && Path::new("data/cost_matrix.csv").exists()
}

#[test]
fn test_regression_qwerty_vs_colemak() {
    if !has_real_data() {
        println!("Skipping regression test: Real data not found in 'data/'");
        return;
    }

    // 1. Run Validation on QWERTY
    let output_qwerty = Command::new("./target/release/keyforge")
        .args([
            // Removed &
            "validate",
            "--layout",
            "qwerty",
            "--geometry",
            "data/szr35.json",
            "--weight-finger-effort",
            "0.5",
            "--weight-geo-dist",
            "3.0",
            "--penalty-sfb-base",
            "200.0",
        ])
        .output()
        .expect("Failed to run QWERTY validation");

    // 2. Run Validation on Colemak
    let output_colemak = Command::new("./target/release/keyforge")
        .args([
            // Removed &
            "validate",
            "--layout",
            "colemak",
            "--geometry",
            "data/szr35.json",
            "--weight-finger-effort",
            "0.5",
            "--weight-geo-dist",
            "3.0",
            "--penalty-sfb-base",
            "200.0",
        ])
        .output()
        .expect("Failed to run Colemak validation");

    let stdout_q = String::from_utf8_lossy(&output_qwerty.stdout);
    let stdout_c = String::from_utf8_lossy(&output_colemak.stdout);

    let score_q = extract_score(&stdout_q);
    let score_c = extract_score(&stdout_c);

    println!("Real Data Regression Check:");
    println!("QWERTY Score:  {}", score_q);
    println!("Colemak Score: {}", score_c);

    assert!(
        score_q > score_c * 3.0,
        "Regression Warning: QWERTY is performing suspiciously well against Colemak"
    );
    assert!(
        score_c != 0.0,
        "Regression Warning: Real scores are zero (possible negative/overflow)"
    );
}

fn extract_score(output: &str) -> f32 {
    for line in output.lines() {
        if line.contains("|") && (line.contains("qwerty") || line.contains("colemak")) {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() > 1 {
                return parts[1].trim().parse().unwrap_or(0.0);
            }
        }
    }
    0.0
}
