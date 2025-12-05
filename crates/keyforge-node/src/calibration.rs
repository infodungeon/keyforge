// ===== keyforge/crates/keyforge-node/src/calibration.rs =====
use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::loader::{CorpusBundle, RawCostData}; // Updated import
use keyforge_core::scorer::{Scorer, ScorerBuildParams};
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tracing::info;

pub fn run_calibration() {
    info!("🔌 Initializing KeyForge Node Calibration...");

    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
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

    // 1. Manually construct CorpusBundle for benchmark
    let mut bundle = CorpusBundle::default();
    let chars = "abcdefghijklmnopqrstuvwxyz.,;/";

    // Fill chars
    for c in chars.chars() {
        if c.is_ascii() {
            bundle.char_freqs[c as usize] = 1000.0;
        }
    }

    // Fill Bigrams
    bundle.bigrams.push((b't', b'h', 5000.0));
    bundle.bigrams.push((b'h', b'e', 4000.0));
    bundle.bigrams.push((b'i', b'n', 3000.0));
    bundle.bigrams.push((b'e', b'r', 3000.0));

    // 2. Cost Matrix
    let cost_data = RawCostData {
        entries: Vec::new(),
    };

    ScorerBuildParams::builder()
        .geometry(geom)
        .weights(ScoringWeights::default())
        .defs(LayoutDefinitions::default())
        .cost_data(cost_data)
        .corpus(bundle)
        .debug(false)
        .build()
        .build_scorer()
        .expect("Failed to build scorer")
}
