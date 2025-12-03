use keyforge_core::config::{LayoutDefinitions, ScoringWeights};
use keyforge_core::geometry::{KeyNode, KeyboardGeometry};
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::{Scorer, ScorerBuilder};
use std::io::Cursor;
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tracing::info;

pub fn run_calibration() {
    info!("🔌 Initializing KeyForge Node Calibration...");

    // Initialize System Info to get CPU details
    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));

    // Wait a bit to get accurate CPU usage readings
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();

    let cpu_count = sys.cpus().len();
    let memory = sys.total_memory() / 1024 / 1024;
    let host_name = System::host_name().unwrap_or("Unknown".into());

    info!("🖥️  Host: {}", host_name);
    info!("🧠  CPU: {} cores", cpu_count);
    info!("💾  RAM: {} MB", memory);

    info!("🚀 Preparing Physics Engine for Stress Test...");
    let scorer = setup_benchmark_scorer();

    // Safety check: ensure geometry has enough keys
    let limit = scorer.key_count.min(30);

    // CHANGED (Phase 2): Convert layout bytes to u16
    let layout_codes: Vec<u16> = b"abcdefghijklmnopqrstuvwxyz.,;/"
        .iter()
        .take(limit)
        .map(|&b| b as u16)
        .collect();

    // CHANGED (Phase 2): pos_map is now Box<[u8; 65536]>
    let pos_map = mutation::build_pos_map(&layout_codes);

    // Warmup phase to let CPU boost clocks settle
    info!("🔥 Warming up...");
    let warmup_iters = 50_000;
    for _ in 0..warmup_iters {
        std::hint::black_box(scorer.score_full(&pos_map, 3000));
    }

    info!("⚡ Running Benchmark (5s)...");
    let start = Instant::now();
    let duration = Duration::from_secs(5);
    let mut iterations: u64 = 0;

    // Hot loop
    while start.elapsed() < duration {
        // Batching to reduce time-check overhead
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

// Helper to build a scorer without reading from disk
fn setup_benchmark_scorer() -> Scorer {
    // 1. Generate a mock 30-key Ortho Geometry
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
        prime_slots: vec![13, 14, 15, 16], // Standard home row indices
        med_slots: vec![1, 2, 3, 4],
        low_slots: vec![20, 21, 22],
        home_row: 1,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();

    // 2. Synthesize N-Gram Data
    let mut ngram_data = String::new();
    let chars = "abcdefghijklmnopqrstuvwxyz.,;/";
    // Monograms
    for c in chars.chars() {
        ngram_data.push_str(&format!("{}\t1000\n", c));
    }
    // Bigrams
    ngram_data.push_str("th\t5000\n");
    ngram_data.push_str("he\t4000\n");
    ngram_data.push_str("in\t3000\n");
    ngram_data.push_str("er\t3000\n");

    let cursor = Cursor::new(ngram_data);
    let weights = ScoringWeights::default();
    let defs = LayoutDefinitions::default();

    // 3. Build Scorer
    ScorerBuilder::new()
        .with_weights(weights)
        .with_defs(defs)
        .with_geometry(geom)
        .with_ngrams_from_reader(cursor)
        .expect("Failed to build bench scorer")
        .build()
        .expect("Failed to build scorer")
}
