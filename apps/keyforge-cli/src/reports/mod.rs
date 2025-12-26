// ===== keyforge/crates/keyforge-cli/src/reports/mod.rs =====
mod benchmarks;
mod grid;
mod tables;

// pub use self::grid::print_layout as print_layout_grid;
// pub use self::tables::{scoring as print_scoring_report, statistical as print_statistical_report};

// CHANGED: scorer -> scoring
#[allow(dead_code)]
pub fn print_comparison_report(results: &[(String, keyforge_model::AnalysisReport)]) {
    let bench_data = benchmarks::load();
    tables::comparisons(results, bench_data.as_ref());
}
