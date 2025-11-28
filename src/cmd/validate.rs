use crate::reports; // This stays 'crate'
use clap::Args;
use keyforge::config::Config;
use keyforge::layouts::get_all_layouts;
use keyforge::optimizer::mutation;
use keyforge::scorer::Scorer;
use std::sync::Arc;

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    #[command(flatten)]
    pub config: Config,

    #[arg(short, long)]
    pub layout: Option<String>,
}

pub fn run(args: ValidateArgs, scorer: Arc<Scorer>) {
    let layouts = get_all_layouts();
    let mut results = Vec::new();
    let eval_limit = args.config.search.opt_limit_slow;

    println!("\n🔎 === LAYOUT AUDIT === 🔎");
    for (layout_enum, layout_bytes) in &layouts {
        let name = layout_enum.to_string();
        if let Some(ref filter) = args.layout {
            if !name.to_lowercase().contains(&filter.to_lowercase()) {
                continue;
            }
        }

        reports::print_layout_grid(&name, layout_bytes);

        let pos_map = mutation::build_pos_map(layout_bytes);
        let details = scorer.score_debug(&pos_map, eval_limit);
        results.push((name, details));
    }

    // Sort by Layout Score
    results.sort_by(|a, b| a.1.layout_score.partial_cmp(&b.1.layout_score).unwrap());

    // Generate Reports
    reports::print_scoring_report(&results);
    reports::print_statistical_report(&results, &scorer.weights);
    reports::print_comparison_report(&results);
}
