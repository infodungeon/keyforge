#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/profile.rs

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

use crate::constants::{DEFAULT_PERSONAL_COST_PATH, DEFAULT_USER_STATS_PATH};
use clap::Args;
use keyforge_compute::biometrics::BiometricProfiler;
use keyforge_model::CostModel;
use keyforge_protocol::BiometricSample;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct ProfileArgs {
    /// Path to the user statistics file (JSON or JSONL)
    #[arg(short, long, default_value = DEFAULT_USER_STATS_PATH)]
    pub input: PathBuf,

    /// Path to write the generated cost profile JSON
    #[arg(short, long, default_value = DEFAULT_PERSONAL_COST_PATH)]
    pub output: PathBuf,
}

pub fn run(args: &ProfileArgs) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🧬 Generating Biometric Profile...");
    eprintln!("   Input:  {}", args.input.display());
    eprintln!("   Output: {}", args.output.display());

    if !args.input.exists() {
        return Err(format!("Input file not found: {}", args.input.display()).into());
    }

    let file = File::open(&args.input).map_err(|e| format!("Failed to open input file: {e}"))?;

    let reader = BufReader::new(file);
    let mut samples = Vec::new();
    let mut error_count = 0;

    for l in reader.lines().map_while(Result::ok) {
        if l.trim().is_empty() {
            continue;
        }
        if let Ok(s) = serde_json::from_str::<BiometricSample>(&l) {
            samples.push(s);
        } else {
            // If it looks like it might be a JSON array, or if it just failed to parse as a single sample,
            // try parsing the whole file as a legacy UserStatsStore.
            if error_count == 0 {
                let content = std::fs::read_to_string(&args.input).unwrap_or_default();
                if let Ok(legacy_store) =
                    serde_json::from_str::<keyforge_protocol::UserStatsStore>(&content)
                {
                    eprintln!(
                        "⚠️  Legacy JSON array format detected. Loading entire file into memory."
                    );
                    samples = legacy_store.biometrics.to_vec();
                    break;
                }
            }
            error_count += 1;
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

    let base_model = CostModel::default();
    let profile = BiometricProfiler::profile(&samples, &base_model);
    let json = serde_json::to_string_pretty(&profile)?;

    std::fs::write(&args.output, json).map_err(|e| format!("Failed to write output file: {e}"))?;

    eprintln!("✅ Profile generated successfully.");
    Ok(())
}
