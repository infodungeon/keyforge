use crate::cli_parsers::resolve_path;
use clap::Args;
use keyforge_infra::fs::io::read_to_string_limited;
use keyforge_protocol::constants::MAX_INPUT_FILE_SIZE;
use keyforge_protocol::geometry::KeyboardDefinition;
use keyforge_protocol::job::JobIdentifier;
use keyforge_protocol::CostMatrixSource;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct QueryArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,

    #[arg(long, default_value = "http://localhost:3000")]
    pub hive: String,
    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

use keyforge_protocol::Validator;

pub async fn run(args: QueryArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🔍 Calculating Job Hash for criteria…");

    let kb_path = resolve_path(&args.shared.keyboard, Some("keyboards"), root)?;

    let kb_content = read_to_string_limited(&kb_path, MAX_INPUT_FILE_SIZE)
        .map_err(|e| format!("Failed to read keyboard file: {}", e))?;

    let kb_def = KeyboardDefinition::parse(&kb_content, None)
        .map_err(|e| format!("Failed to parse keyboard definition: {}", e))?;

    let corpora_fingerprint = "default".to_string();
    let constraints = args.shared.pinned_keys;

    use std::convert::TryFrom;
    let config = keyforge_protocol::config::Config::try_from(args.config)?;
    config.search.validate()?;
    config.weights.validate()?;

    let cost_source = CostMatrixSource::Predefined(args.shared.cost.clone());

    let job_id = JobIdentifier::try_from_parts(
        &kb_def.geometry,
        &config.weights,
        &config.search,
        &constraints,
        &corpora_fingerprint,
        &cost_source,
    )
    .map_err(|e| format!("Failed to compute job id: {}", e))?
    .hash;

    eprintln!("   Job ID: {}", job_id);
    eprintln!("   Hive:   {}", args.hive);

    let url = format!("{}/jobs/{}/population", args.hive, job_id);

    match reqwest::get(&url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                let pop = resp
                    .json::<keyforge_protocol::PopulationResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse Hive response: {}", e))?;

                eprintln!("\n✅ Job Found!");
                if let Some(best) = pop.layouts.first() {
                    println!("{}", best);
                } else {
                    eprintln!("   Job exists but has no results yet.");
                }
            } else {
                return Err(format!("Hive Error: {}", resp.status()).into());
            }
        }
        Err(e) => return Err(format!("Connection Failed: {}", e).into()),
    }
    Ok(())
}
