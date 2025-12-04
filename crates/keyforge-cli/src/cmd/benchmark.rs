use clap::Args;
use keyforge_core::config::Config;
use keyforge_core::optimizer::Replica;
use keyforge_core::scorer::Scorer;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Args, Debug, Clone)]
pub struct BenchmarkArgs {
    #[command(flatten)]
    pub config: Config,

    // Kept for compatibility, but we override with time-based logic below
    #[arg(long, default_value_t = 1_000_000)]
    pub iterations: usize,
}

pub fn run(_args: BenchmarkArgs, scorer: Arc<Scorer>) {
    info!("🏎️  Starting 2-Minute Bench Race...");
    info!("    Target: 60s Warmup + 60s Measurement");
    info!("    Engine: Quantized Replica (L2 Cache Optimized)");

    // 1. Initialize a Replica
    // This triggers the quantization (f32 -> i16) and allocates the compact vectors
    let mut replica = Replica::new(
        scorer.clone(),
        100.0,    // Temperature
        Some(42), // Seed
        100,      // Fast limit
        100,      // Slow limit
        "",       // No pins
    );

    // Batch size for the loop (amortize time checking overhead)
    let batch_size = 5000;

    // --- PHASE 1: WARMUP (60s) ---
    info!("🔥 Phase 1: Warmup (Stabilizing CPU Boost)...");
    let warmup_start = Instant::now();
    let warmup_duration = Duration::from_secs(60);

    let mut warmup_ops = 0;
    while warmup_start.elapsed() < warmup_duration {
        replica.evolve(batch_size);
        warmup_ops += batch_size;
    }

    let warmup_rate = warmup_ops as f64 / warmup_start.elapsed().as_secs_f64();
    info!("    Warmup Rate: {:.2} M/s", warmup_rate / 1_000_000.0);

    // --- PHASE 2: MEASUREMENT (60s) ---
    info!("⏱️  Phase 2: Measurement (Recording)...");
    let measure_start = Instant::now();
    let measure_duration = Duration::from_secs(60);

    let mut total_ops: u64 = 0;

    while measure_start.elapsed() < measure_duration {
        replica.evolve(batch_size);
        total_ops += batch_size as u64;
    }

    let elapsed = measure_start.elapsed().as_secs_f64();
    let ops_per_sec = total_ops as f64 / elapsed;

    info!("🏁 Benchmark Complete");
    info!("========================================");
    info!("    Total Ops:   {}", total_ops);
    info!("    Time:        {:.4}s", elapsed);
    info!(
        "    Throughput:  {:.3} Million Ops/sec",
        ops_per_sec / 1_000_000.0
    );
    info!("========================================");

    if ops_per_sec < 5_000_000.0 {
        info!("⚠️  Note: Throughput seems low for an i7. Ensure you are running with --release");
    }
}
