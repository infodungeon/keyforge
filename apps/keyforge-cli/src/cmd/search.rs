// apps/keyforge-cli/src/cmd/search.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use keyforge_model::config::Config;
use keyforge_protocol::{JobConfig, ResultSubmission};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::NamedTempFile;
use crate::error::CliError;

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[arg(short = 'T', long)]
    pub time: Option<u64>,

    #[arg(short = 'a', long)]
    pub attempts: Option<usize>,

    #[arg(short = 'S', long)]
    pub seed: Option<u64>,

    #[arg(long, default_value_t = 0)]
    pub threads: usize,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

pub fn run(args: SearchArgs, root: &Path) -> Result<(), CliError> {
    eprintln!("🔎 Starting optimisation (via Agent Sidecar)…");

    use std::convert::TryFrom;
    let mut model_config = Config::try_from(args.config.clone())
        .map_err(|e| CliError::Config(e))?;

    // Apply Overrides
    if let Some(s) = args.seed {
        // [Fixed] Directly access struct field
        model_config.search.seed = Some(s);
    }

    // Resolve Assets (Keyboard, Cost, etc)
    let kb_input = args.shared.keyboard.unwrap_or_else(|| "ortho_30".into());
    let kb_path = crate::cli_parsers::resolve_path(&kb_input, Some("keyboards"), root)
        .map_err(|e| CliError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    let kb_content = std::fs::read_to_string(&kb_path)?;
    let kb_def = keyforge_model::geometry::KeyboardDefinition::parse(&kb_content, None)
        .map_err(|e| CliError::Config(e))?;

    let corpus_sources = crate::cli_args::parse_corpora(&args.shared.corpus.unwrap_or_default())
        .map_err(|e| CliError::Config(e))?;

    let job = JobConfig {
        definition: kb_def,
        weights: model_config.weights,
        params: model_config.search,
        pinned_keys: args.shared.pinned_keys,
        corpora: corpus_sources,
        cost_matrix: keyforge_model::CostMatrixSource::Predefined(
            args.shared.cost.unwrap_or_else(|| "default_costmatrix.json".into())
        ),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    let job_file = NamedTempFile::new()?;
    serde_json::to_writer(&job_file, &job)?;
    let job_path = job_file.path().to_path_buf();

    let mut cmd = Command::new("keyforge-agent");
    cmd.arg("run")
       .arg(&job_path)
       .env("KEYFORGE_DATA_DIR", root) 
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        CliError::Other(format!("Failed to spawn keyforge-agent: {}. Is it installed?", e))
    })?;

    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);
    
    // [Fixed] Direct access to struct field
    let steps = job.params.search_steps;
    
    let pb = ProgressBar::new(steps as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();

    std::thread::spawn(move || {
        for line in reader.lines() {
            if let Ok(l) = line {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&l) {
                    if let Some(fields) = json.get("fields") {
                        if let Some(step) = fields.get("step").and_then(|v| v.as_u64()) {
                            pb.set_position(step);
                            if let Some(score) = fields.get("score").and_then(|v| v.as_f64()) {
                                pb.set_message(format!("Score: {:.2}", score));
                            }
                        }
                    }
                }
            }
            if !r_clone.load(Ordering::Relaxed) { break; }
        }
        pb.finish_with_message("Done");
    });

    let output = child.wait_with_output()?;
    running.store(false, Ordering::Relaxed);

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::Other(format!("Agent failed: {}", err)));
    }

    let result_json = String::from_utf8_lossy(&output.stdout);
    let submission: ResultSubmission = serde_json::from_str(&result_json)?;

    eprintln!("\n=== FINAL RESULT ===");
    eprintln!("Score: {:.3}", submission.score);
    
    if let Ok(_layout_obj) = serde_json::from_str::<keyforge_model::Layout>(&submission.layout) {
         println!("{}", submission.layout);
    } else {
         println!("{}", submission.layout);
    }

    Ok(())
}
