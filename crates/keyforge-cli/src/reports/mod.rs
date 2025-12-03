mod benchmarks;
mod grid;
mod tables;

pub use self::grid::print_layout as print_layout_grid;
pub use self::tables::{scoring as print_scoring_report, statistical as print_statistical_report};

// We wrap the comparison logic here to handle loading internally or passing data
pub fn print_comparison_report(results: &[(String, keyforge_core::scorer::ScoreDetails)]) {
    let bench_data = benchmarks::load();
    tables::comparisons(results, bench_data.as_ref());
}
