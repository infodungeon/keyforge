use crate::reports;
use clap::Args;
use keyforge_core::config::Config;
use keyforge_core::geometry::KeyboardDefinition;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16;
use keyforge_core::scorer::Scorer;
use keyforge_core::verifier::Verifier;
use std::sync::Arc;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: Config,

    #[arg(short, long)]
    pub layout: Option<String>,
}

pub fn run(
    args: ValidateArgs,
    kb_def: &KeyboardDefinition,
    scorer: Arc<Scorer>,
    registry: Arc<KeycodeRegistry>,
) {
    let mut results = Vec::new();
    let key_count = kb_def.geometry.keys.len();

    // 1. Create Verifier wrapper
    let verifier = Verifier::from_components(scorer.clone(), registry.clone());

    let mut sorted_names: Vec<_> = kb_def.layouts.keys().collect();
    sorted_names.sort();

    println!("\n🔎 === LAYOUT AUDIT: {} === 🔎", kb_def.meta.name);

    for name in sorted_names {
        if let Some(ref filter) = args.layout {
            if !name.to_lowercase().contains(&filter.to_lowercase()) {
                continue;
            }
        }

        let layout_str = kb_def.layouts.get(name).unwrap();

        // 2. Score via Verifier (Standardized Logic)
        // FIXED: Removed extra argument, Verifier handles limits internally
        let details = verifier.score_details(layout_str.clone());

        // 3. Print Visual Grid
        let layout_codes = layout_string_to_u16(layout_str, key_count, &registry);
        reports::print_layout_grid(name, &layout_codes, &registry);

        results.push((name.clone(), details));
    }

    if results.is_empty() {
        println!("No layouts found matching criteria.");
        return;
    }

    results.sort_by(|a, b| a.1.layout_score.partial_cmp(&b.1.layout_score).unwrap());

    reports::print_scoring_report(&results);
    reports::print_statistical_report(&results, &scorer.weights);
    reports::print_comparison_report(&results);
}
