use clap::Args;
use keyforge_protocol::BiometricSample;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct ProfileArgs {
    /// Path to the user statistics file (JSON or JSONL)
    #[arg(short, long, default_value = "data/user_stats.jsonl")]
    pub input: PathBuf,

    /// Path to write the generated cost matrix CSV
    #[arg(short, long, default_value = "data/personal_cost.json")]
    pub output: PathBuf,
}

pub fn run(args: ProfileArgs) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🧬 Generating Biometric Profile...");
    eprintln!("   Input:  {:?}", args.input);
    eprintln!("   Output: {:?}", args.output);

    if !args.input.exists() {
        return Err(format!("Input file not found: {:?}", args.input).into());
    }

    let file = File::open(&args.input).map_err(|e| format!("Failed to open input file: {}", e))?;

    let reader = BufReader::new(file);
    let mut samples = Vec::new();
    let mut error_count = 0;

    for l in reader.lines().map_while(Result::ok) {
        if l.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<BiometricSample>(&l) {
            Ok(s) => samples.push(s),
            Err(_) => {
                if l.trim().starts_with('[') {
                    eprintln!(
                        "⚠️  Legacy JSON array format detected. Loading entire file into memory."
                    );
                    let content = std::fs::read_to_string(&args.input).unwrap_or_default();
                    if let Ok(legacy_store) =
                        serde_json::from_str::<keyforge_protocol::UserStatsStore>(&content)
                    {
                        samples = legacy_store.biometrics;
                        break;
                    }
                }
                error_count += 1;
            }
        }
    }

    if samples.is_empty() {
        eprintln!("⚠️  Warning: No valid biometric samples found.");
    } else {
        eprintln!(
            "   Loaded {} samples. (Skipped {} errors)",
            samples.len(),
            error_count
        );
    }

    let store = keyforge_protocol::UserStatsStore {
        sessions: 1,
        total_keystrokes: samples.len() as u64,
        biometrics: samples,
    };

    let csv_content = keyforge_infra::util::common::generate_cost_matrix_from_stats(&store);

    std::fs::write(&args.output, csv_content)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    eprintln!("✅ Profile generated successfully.");
    Ok(())
}
