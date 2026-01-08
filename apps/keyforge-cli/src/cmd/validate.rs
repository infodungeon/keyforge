// apps/keyforge-cli/src/cmd/validate.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use clap::Args;
use keyforge_compute::Runtime;
use keyforge_model::KeyCode;
use std::error::Error;

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

pub fn run(args: ValidateArgs, runtime: Runtime) -> Result<(), Box<dyn Error>> {
    eprintln!("🔎 Validating Layout...");

    if let Some(_name) = &args.layout {
        return Err("Looking up layouts by name is temporarily disabled in v0.8.0. Please provide the layout string directly.".into());
    }

    // Use the keys from the registry to form a dummy layout for testing
    // In a real scenario, we would parse a layout string here.
    let key_count = runtime.engine.key_count();
    let dummy_layout = keyforge_model::Layout::new_unchecked((0..key_count).map(|i| KeyCode(i as u16)).collect());

    let report = runtime.analyze(&dummy_layout)?;

    eprintln!("=== Analysis Report (Dummy) ===");
    eprintln!("Score:        {:.3}", report.score);
    eprintln!("Distance:     {:.3}", report.distance);
    eprintln!("SFB Ratio:    {:.2}%", report.sfb_ratio * 100.0);
    eprintln!("Hand Balance: {:.2}", report.hand_balance);

    Ok(())
}
