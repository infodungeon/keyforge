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

    let layout = if let Some(name) = &args.layout {
        // If it looks like a layout string (contains spaces), parse it.
        if name.contains(' ') {
             keyforge_adapter::conversion::parse_layout_string(name, runtime.engine.key_count(), &runtime.registry)?
        } else {
             return Err(format!("Layout lookup by name '{}' is not supported yet. Please provide the full layout string.", name).into());
        }
    } else {
        // Use the keys from the registry to form a dummy layout for testing
        let key_count = runtime.engine.key_count();
        keyforge_model::Layout::new_unchecked((0..key_count).map(|i| KeyCode(i as u16)).collect())
    };

    let report = runtime.analyze(&layout)?;

    eprintln!("=== Analysis Report ===");
    eprintln!("Score:        {:.3}", report.score);
    eprintln!("Distance:     {:.3}", report.distance);
    eprintln!("SFB Ratio:    {:.2}%", report.sfb_ratio * 100.0);
    eprintln!("Hand Balance: {:.2}", report.hand_balance);

    Ok(())
}
