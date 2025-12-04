use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::{Scorer, ScorerBuildParams};
use std::io::Cursor;
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tracing::info;

pub fn run_calibration() {
    info!("🔌 Initializing KeyForge Node Calibration...");

    // FIXED: Use RefreshKind::new() instead of nothing()
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

    // FIXED: Use refresh_cpu() instead of refresh_cpu_all()
    sys.refresh_cpu();

    let cpu_count = sys.cpus().len();
    let memory = sys.total_memory() / 1024 / 1024;
    let host_name = System::host_name().unwrap_or("Unknown".into());

    info!("🖥️  Host: {}", host_name);
    info!("🧠  CPU: {} cores", cpu_count);
    info!("💾  RAM: {} MB", memory);

    info!("🚀 Preparing Physics Engine for Stress Test...");
    let scorer = setup_benchmark_scorer();

    let limit = scorer.key_count.min(30);
    let layout_codes: Vec<u16> = b"abcdefghijklmnopqrstuvwxyz.,;/"
        .iter()
        .take(limit)
        .map(|&b| b as u16)
        .collect();

    let pos_map = mutation::build_pos_map(&layout_codes);

    info!("🔥 Warming up...");
    let warmup_iters = 50_000;
    for _ in 0..warmup_iters {
        std::hint::black_box(scorer.score_full(&pos_map, 3000));
    }

    info!("⚡ Running Benchmark (5s)...");
    let start = Instant::now();
    let duration = Duration::from_secs(5);
    let mut iterations: u64 = 0;

    while start.elapsed() < duration {
        for _ in 0..100 {
            std::hint::black_box(scorer.score_full(&pos_map, 3000));
        }
        iterations += 100;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let sops = iterations as f64 / elapsed;

    info!("✅ Calibration Complete");
    info!(
        "🚀 Performance: {:.2} Million Evaluations/sec (Single Core)",
        sops / 1_000_000.0
    );

    if sops < 1_000_000.0 {
        info!("⚠️  Note: Performance is lower than expected. Ensure you are running in --release mode.");
    }
}

fn setup_benchmark_scorer() -> Scorer {
    let mut keys = Vec::new();
    for r in 0..3 {
        for c in 0..10 {
            keys.push(KeyNode {
                id: format!("k_{}_{}", r, c),
                hand: if c < 5 { 0 } else { 1 },
                finger: (c % 5) as u8,
                row: r as i8,
                col: c as i8,
                x: c as f32,
                y: r as f32,
                w: 1.0,
                h: 1.0,
                is_stretch: false,
            });
        }
    }

    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: vec![13, 14, 15, 16],
        med_slots: vec![1, 2, 3, 4],
        low_slots: vec![20, 21, 22],
        home_row: 1,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    let mut ngram_data = String::new();
    let chars = "abcdefghijklmnopqrstuvwxyz.,;/";
    for c in chars.chars() {
        ngram_data.push_str(&format!("{}\t1000\n", c));
    }
    ngram_data.push_str("th\t5000\n");
    ngram_data.push_str("he\t4000\n");
    ngram_data.push_str("in\t3000\n");
    ngram_data.push_str("er\t3000\n");

    let cursor = Cursor::new(ngram_data);
    let weights = ScoringWeights::default();

    let cost_cursor = Cursor::new("From,To,Cost\n");

    ScorerBuildParams::from_readers(
        cost_cursor,
        cursor,
        geom,
        Some(weights),
        Some(LayoutDefinitions::default()),
        false,
    )
    .expect("Failed to build scorer")
}
