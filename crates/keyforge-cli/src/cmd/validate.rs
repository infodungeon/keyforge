use crate::reports;
use clap::Args;
use keyforge_core::config::Config;
use keyforge_core::geometry::KeyboardDefinition;
use keyforge_core::layouts::layout_string_to_bytes;
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::Scorer;
use std::sync::Arc;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: Config,

    #[arg(short, long)]
    pub layout: Option<String>,
}

pub fn run(args: ValidateArgs, kb_def: &KeyboardDefinition, scorer: Arc<Scorer>) {
    let mut results = Vec::new();
    let eval_limit = args.config.search.opt_limit_slow;
    let key_count = kb_def.geometry.keys.len();

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
        let layout_bytes = layout_string_to_bytes(layout_str, key_count);

        reports::print_layout_grid(name, &layout_bytes);

        let pos_map = mutation::build_pos_map(&layout_bytes);

        // RENAMED from score_debug
        let details = scorer.score_details(&pos_map, eval_limit);

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
