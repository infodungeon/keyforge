use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::sync::Mutex;

pub struct SystemMonitor {
    sys: Mutex<System>,
    cpu_usage: AtomicU32,
    memory_used: AtomicU64,
    uptime: AtomicU64,
    total_ops: AtomicU64,
    last_ops: AtomicU64,
    ops_per_sec: AtomicU64, // f64 bits
    last_refresh: Mutex<Instant>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            sys: Mutex::new(System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            )),
            cpu_usage: AtomicU32::new(0),
            memory_used: AtomicU64::new(0),
            uptime: AtomicU64::new(0),
            total_ops: AtomicU64::new(0),
            last_ops: AtomicU64::new(0),
            ops_per_sec: AtomicU64::new(0),
            last_refresh: Mutex::new(Instant::now()),
        }
    }

    /// Refresh the internal sysinfo state and update atomic metrics.
    /// This requires an internal lock on sysinfo::System.
    pub async fn refresh(&self) {
        let mut sys = self.sys.lock().await;
        sys.refresh_all();
        
        let cpu = sys.global_cpu_info().cpu_usage();
        let mem = sys.used_memory();
        let up = System::uptime();

        self.cpu_usage.store(cpu.to_bits(), Ordering::Relaxed);
        self.memory_used.store(mem, Ordering::Relaxed);
        self.uptime.store(up, Ordering::Relaxed);

        // Update throughput
        let mut last_ref_lock = self.last_refresh.lock().await;
        let elapsed = last_ref_lock.elapsed().as_secs_f64();
        
        if elapsed >= 1.0 {
            let current_ops = self.total_ops.load(Ordering::Relaxed);
            let prev_ops = self.last_ops.swap(current_ops, Ordering::Relaxed);
            let delta = current_ops.saturating_sub(prev_ops);
            let rate = (delta as f64) / elapsed;
            
            self.ops_per_sec.store(rate.to_bits(), Ordering::Relaxed);
            *last_ref_lock = Instant::now();
        }
    }

    /// Increment the total operations counter.
    /// Lock-free.
    pub fn record_op(&self) {
        self.total_ops.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_total_ops(&self) -> u64 {
        self.total_ops.load(Ordering::Relaxed)
    }

    pub fn get_ops_per_sec(&self) -> f64 {
        f64::from_bits(self.ops_per_sec.load(Ordering::Relaxed))
    }

    pub fn get_memory_used(&self) -> u64 {
        self.memory_used.load(Ordering::Relaxed)
    }

    pub fn get_cpu_usage(&self) -> f32 {
        f32::from_bits(self.cpu_usage.load(Ordering::Relaxed))
    }

    pub fn get_uptime(&self) -> u64 {
        self.uptime.load(Ordering::Relaxed)
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedMonitor = Arc<SystemMonitor>;
