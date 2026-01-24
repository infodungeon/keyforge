#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/query.rs

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

use crate::cli_parsers::resolve_path;
use crate::constants::DEFAULT_HIVE_URL;
use clap::Args;
use keyforge_infra::fs::io::read_to_string_limited;
use keyforge_model::constants::{
    ASSET_DEFAULT_COST_MATRIX, DEFAULT_KEYBOARD_ID, MAX_INPUT_FILE_SIZE,
};
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::job::JobIdentifier;
use keyforge_model::CostMatrixSource;
use std::convert::TryFrom;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct QueryArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[arg(long, default_value = DEFAULT_HIVE_URL)]
    pub hive: String,
    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

use keyforge_model::Validator;

pub async fn run(args: QueryArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🔍 Calculating Job Hash for criteria…");

    // Resolve Defaults
    let kb_input = args
        .shared
        .keyboard
        .unwrap_or_else(|| DEFAULT_KEYBOARD_ID.to_string());
    let cost_input = args
        .shared
        .cost
        .unwrap_or_else(|| ASSET_DEFAULT_COST_MATRIX.to_string());

    let kb_path = resolve_path(&kb_input, Some("keyboards"), root)?;

    let kb_content = read_to_string_limited(&kb_path, MAX_INPUT_FILE_SIZE)
        .map_err(|e| format!("Failed to read keyboard file: {e}"))?;

    let kb_def = KeyboardDefinition::parse(&kb_content, None)
        .map_err(|e| format!("Failed to parse keyboard definition: {e}"))?;

    let corpora_input = args
        .shared
        .corpus
        .unwrap_or_else(|| vec!["text/en_std".to_string()]);
    let mut domain_corpora = Vec::new();
    for s in corpora_input {
        domain_corpora.push(s.parse::<keyforge_model::config::CorpusSource>()?);
    }
    let corpora_fingerprint = keyforge_infra::util::common::calculate_fingerprint(&domain_corpora);
    let constraints = args.shared.pinned_keys;

    let config = keyforge_model::config::Config::try_from(args.config)?;
    config.search.validate()?;
    config.weights.validate()?;

    let cost_source = CostMatrixSource::Predefined(cost_input);

    let proto_geometry: keyforge_model::geometry::KeyboardGeometry =
        serde_json::from_value(serde_json::to_value(&kb_def.geometry)?)?;

    let job_id = JobIdentifier::try_from_parts(
        &proto_geometry,
        &config.weights,
        &config.search,
        &constraints,
        &corpora_fingerprint,
        &cost_source,
    )
    .map_err(|e| format!("Failed to compute job id: {e}"))?
    .hash;

    eprintln!("   Job ID: {job_id}");
    eprintln!("   Hive:   {}", args.hive);

    let url = format!("{}/jobs/{}/population", args.hive, job_id);

    match reqwest::get(&url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                let pop = resp
                    .json::<keyforge_protocol::PopulationResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse Hive response: {e}"))?;

                eprintln!("\n✅ Job Found!");
                if let Some(best) = pop.layouts.first() {
                    println!("{best}");
                } else {
                    eprintln!("   Job exists but has no results yet.");
                }
            } else {
                return Err(format!("Hive Error: {}", resp.status()).into());
            }
        }
        Err(e) => return Err(format!("Connection Failed: {e}").into()),
    }
    Ok(())
}
