#![allow(clippy::print_stdout, clippy::print_stderr)]
// apps/keyforge-cli/src/cmd/export.rs

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

use clap::{Args, Subcommand, ValueEnum};
use keyforge_adapter::loader::AssetLoader;
use keyforge_export::{qmk::QmkExporter, via::ViaExporter, zmk::ZmkExporter, Exporter};
use keyforge_infra::FsProvider;
use keyforge_model::constants::ASSET_KEYCODES;
use keyforge_model::geometry::kle::to_kle_json;
use keyforge_model::geometry::KeyboardDefinition;
use keyforge_model::keycodes::KeycodeRegistry;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct ExportArgs {
    #[command(subcommand)]
    pub command: ExportCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ExportCommands {
    Firmware {
        #[arg(short, long)]
        keyboard: String,
        #[arg(short, long)]
        layout: String,
        #[arg(short, long, value_enum)]
        format: FirmwareFormat,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum FirmwareFormat {
    Qmk,
    Zmk,
    Via,
    Kle,
}

pub async fn run(args: ExportArgs, loader: &FsProvider) -> Result<(), Box<dyn Error>> {
    match args.command {
        ExportCommands::Firmware {
            keyboard,
            layout,
            format,
            output,
        } => {
            eprintln!("💾 Exporting '{layout}' to {format:?}...");

            let def = loader
                .load::<KeyboardDefinition>(&keyboard)
                .await
                .map_err(|e| format!("Failed to load keyboard '{keyboard}': {e}"))?;

            // Load Layout Catalog
            let catalog = loader
                .load::<keyforge_model::layout::LayoutCatalog>(&keyboard)
                .await
                .map_err(|e| format!("Failed to load layout catalog for '{keyboard}': {e}"))?;

            let Some(layout_str) = catalog.layouts.get(&layout) else {
                return Err(
                    format!("Layout '{layout}' not found in catalog for '{keyboard}'.").into(),
                );
            };

            let keys: Vec<String> = layout_str
                .split_whitespace()
                .map(std::string::ToString::to_string)
                .collect();

            // Load Keycode Registry for data-driven export
            let registry = loader.load::<KeycodeRegistry>(ASSET_KEYCODES).await.ok();

            let code = if let FirmwareFormat::Kle = format {
                // Special handling for KLE: Merge layout legends into geometry
                let mut geom = def.geometry.clone();
                if geom.keys.len() != keys.len() {
                    eprintln!(
                        "⚠️  Warning: Layout key count ({}) does not match geometry key count ({}). Export may be incorrect.",
                        keys.len(),
                        geom.keys.len()
                    );
                }
                for (i, key) in geom.keys.iter_mut().enumerate() {
                    if let Some(legend) = keys.get(i) {
                        key.label.clone_from(legend);
                    }
                }
                to_kle_json(&geom)?
            } else {
                let exporter: Box<dyn Exporter> = match format {
                    FirmwareFormat::Qmk => Box::new(QmkExporter),
                    FirmwareFormat::Zmk => Box::new(ZmkExporter),
                    FirmwareFormat::Via => Box::new(ViaExporter),
                    FirmwareFormat::Kle => unreachable!(),
                };
                exporter.generate(&layout, &[keys], registry.as_deref())?
            };

            if let Some(out_path) = output {
                if out_path.exists() {
                    eprintln!(
                        "⚠️  Warning: Output file {} already exists. Overwriting...",
                        out_path.display()
                    );
                }
                fs::write(&out_path, code).map_err(|e| {
                    format!("Failed to write export to {}: {e}", out_path.display())
                })?;
                eprintln!("✅ Exported to {}", out_path.display());
            } else {
                println!("{code}");
            }
            Ok(())
        }
    }
}
