// apps/keyforge-cli/src/cmd/validate.rs

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
use keyforge_model::config::Config;
use keyforge_protocol::JobConfig;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use crate::error::CliError;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,

    /// Name of the layout to validate (must exist in the keyboard definition)
    #[arg(long)]
    pub layout: Option<String>,
}

pub fn run(args: ValidateArgs, root: &Path) -> Result<(), Box<dyn Error>> {
    eprintln!("�� Validating Layout...");

    // 1. Construct Job Config
    use std::convert::TryFrom;
    let model_config = Config::try_from(args.config.clone())
        .map_err(|e| CliError::Config(e))?;

    let kb_input = args.shared.keyboard.clone().unwrap_or_else(|| "ortho_30".into());
    let kb_path = crate::cli_parsers::resolve_path(&kb_input, Some("keyboards"), root)?;
    let kb_content = std::fs::read_to_string(&kb_path)?;
    let kb_def = keyforge_model::geometry::KeyboardDefinition::parse(&kb_content, None)
        .map_err(|e| CliError::Config(e))?;

    let corpus_sources = crate::cli_args::parse_corpora(&args.shared.corpus.clone().unwrap_or_default())
        .map_err(|e| CliError::Config(e))?;

    let job = JobConfig {
        definition: kb_def,
        weights: model_config.weights,
        params: model_config.search,
        pinned_keys: args.shared.pinned_keys.clone(),
        corpora: corpus_sources,
        cost_matrix: keyforge_model::CostMatrixSource::Predefined(
            args.shared.cost.clone().unwrap_or_else(|| "default_costmatrix.json".into())
        ),
        biometrics: vec![],
        parent_job_id: None,
        baseline_score: None,
        parents: vec![],
    };

    // 2. Determine Layout String
    let layout_str = if let Some(name) = &args.layout {
        if name.contains(' ') {
             name.clone()
        } else {
             return Err(format!("Layout lookup by name '{}' is not supported yet.", name).into());
        }
    } else {
        // Dummy layout (0..N)
        let count = job.definition.geometry.keys.len();
        (0..count).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")
    };

    // 3. Serialize to Temp File
    let job_file = NamedTempFile::new()?;
    serde_json::to_writer(&job_file, &job)?;
    let job_path = job_file.path().to_path_buf();

    // 4. Spawn Agent
    let mut cmd = Command::new("keyforge-agent");
    cmd.arg("score")
       .arg("--job-file")
       .arg(&job_path)
       .arg("--layout")
       .arg(layout_str)
       .env("KEYFORGE_DATA_DIR", root)
       .stdout(Stdio::piped())
       .stderr(Stdio::inherit()); // Let user see logs

    let output = cmd.output().map_err(|e| {
        CliError::Other(format!("Failed to spawn keyforge-agent: {}", e))
    })?;

    if !output.status.success() {
        return Err(format!("Agent failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    // 5. Parse and Print Report
    let json = String::from_utf8_lossy(&output.stdout);
    let report: keyforge_model::AnalysisReport = serde_json::from_str(&json)
        .map_err(|e| format!("Invalid JSON from agent: {}", e))?;

    eprintln!("=== Analysis Report ===");
    eprintln!("Score:        {:.3}", report.score);
    eprintln!("Distance:     {:.3}", report.distance);
    eprintln!("SFB Ratio:    {:.2}%", report.sfb_ratio * 100.0);
    eprintln!("Hand Balance: {:.2}", report.hand_balance);

    Ok(())
}
