use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuCacheTopology {
    pub model: String,
    pub architecture: String,
    pub cores: usize,
    pub l1_data_kb: Option<usize>,
    pub l2_kb: Option<usize>,
    pub l3_kb: Option<usize>,
}

impl Default for CpuCacheTopology {
    fn default() -> Self {
        Self {
            model: "Unknown".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cores: 1,
            l1_data_kb: None,
            l2_kb: None,
            l3_kb: None,
        }
    }
}

pub fn detect_topology() -> CpuCacheTopology {
    let mut topo = CpuCacheTopology::default();

    // 1. Basic Info via sysinfo
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    // Wait a moment for CPU usage calculation (though not needed for model name)
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu();

    if let Some(cpu) = sys.cpus().first() {
        topo.model = cpu.brand().trim().to_string();
    }
    topo.cores = sys.cpus().len();

    // 2. Low-Level Cache Detection (x86_64)
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        detect_x86_caches(&mut topo);
    }

    // 3. Low-Level Cache Detection (macOS ARM)
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        detect_apple_silicon_caches(&mut topo);
    }

    info!(
        "🧠 Hardware Detected: {} ({} cores)",
        topo.model, topo.cores
    );
    if let Some(l2) = topo.l2_kb {
        info!("   L2 Cache: {} KB (Critical for Trigram Table)", l2);
    } else {
        warn!("   L2 Cache: Unknown (Will use safe defaults)");
    }

    topo
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn detect_x86_caches(topo: &mut CpuCacheTopology) {
    use raw_cpuid::{CacheType, CpuId};
    let cpuid = CpuId::new();

    if let Some(caches) = cpuid.get_cache_parameters() {
        for cache in caches {
            let size_kb =
                (cache.sets() * cache.associativity() * cache.coherency_line_size()) / 1024;
            match cache.level() {
                1 => {
                    if cache.cache_type() == CacheType::Data {
                        topo.l1_data_kb = Some(size_kb);
                    }
                }
                2 => topo.l2_kb = Some(size_kb),
                3 => topo.l3_kb = Some(size_kb),
                _ => {}
            }
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn detect_apple_silicon_caches(topo: &mut CpuCacheTopology) {
    use std::process::Command;

    let get_sysctl = |name: &str| -> Option<usize> {
        let output = Command::new("sysctl").arg("-n").arg(name).output().ok()?;
        let s = String::from_utf8_lossy(&output.stdout);
        s.trim().parse::<usize>().ok()
    };

    if let Some(bytes) = get_sysctl("hw.l1dcachesize") {
        topo.l1_data_kb = Some(bytes / 1024);
    }
    if let Some(bytes) = get_sysctl("hw.l2cachesize") {
        topo.l2_kb = Some(bytes / 1024);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_topology_is_safe() {
        let topo = CpuCacheTopology::default();
        assert_eq!(topo.cores, 1); // Must default to at least 1 core
        assert_eq!(topo.model, "Unknown");
    }

    #[test]
    fn test_detection_does_not_panic() {
        // This test runs the actual detection on the host.
        // We can't assert values (since CI machines differ),
        // but we can assert it doesn't crash.
        let topo = detect_topology();
        assert!(topo.cores > 0, "System reported 0 cores");
        println!("Detected: {:?}", topo);
    }
}
