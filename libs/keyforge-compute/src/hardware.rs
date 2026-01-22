use keyforge_physics::IntelEngineConfig;
use raw_cpuid::{CacheType, CpuId};

#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub vendor: String,
    pub cache_line_size: u16,
    pub l1d_size_bytes: usize,
    pub l2_size_bytes: usize,
    pub l3_size_bytes: usize,
}

impl From<CpuTopology> for IntelEngineConfig {
    fn from(topo: CpuTopology) -> Self {
        Self {
            l1d_size_bytes: topo.l1d_size_bytes,
            l2_size_bytes: topo.l2_size_bytes,
            l3_size_bytes: topo.l3_size_bytes,
            use_prefetch: true,
        }
    }
}

impl Default for CpuTopology {
    fn default() -> Self {
        Self {
            vendor: "Unknown".to_string(),
            cache_line_size: 64,
            l1d_size_bytes: 32 * 1024,      // 32 KiB Safe default
            l2_size_bytes: 256 * 1024,      // 256 KiB
            l3_size_bytes: 8 * 1024 * 1024, // 8 MiB
        }
    }
}

#[derive(Debug)]
pub struct HardwareProbe;

impl HardwareProbe {
    #[must_use]
    pub fn probe() -> CpuTopology {
        let cpuid = CpuId::new();
        let vendor = cpuid
            .get_vendor_info()
            .map_or_else(|| "Unknown".to_string(), |v| v.as_str().to_string());

        let mut topology = CpuTopology {
            vendor,
            ..Default::default()
        };

        if let Some(cparams) = cpuid.get_cache_parameters() {
            for cache in cparams {
                let size = (cache.associativity() + 1)
                    * (cache.physical_line_partitions() + 1)
                    * (cache.coherency_line_size() + 1)
                    * (cache.sets() + 1);

                match (cache.level(), cache.cache_type()) {
                    (1, CacheType::Data) => {
                        topology.l1d_size_bytes = size;
                        topology.cache_line_size =
                            u16::try_from(cache.coherency_line_size() + 1).unwrap_or(64);
                    }
                    (2, _) => topology.l2_size_bytes = size,
                    (3, _) => topology.l3_size_bytes = size,
                    _ => {}
                }
            }
        }

        topology
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_probe_runs() {
        let topology = HardwareProbe::probe();
        println!("Detected Topology: {topology:?}");
        assert!(!topology.vendor.is_empty());
        // On some CI environments cpuid might be masked, but usually vendor is present.
    }

    #[test]
    fn test_cpu_topology_defaults_and_conversion() {
        let default = CpuTopology::default();
        assert_eq!(default.vendor, "Unknown");

        let config: IntelEngineConfig = default.into();
        assert_eq!(config.l1d_size_bytes, 32 * 1024);
        assert!(config.use_prefetch);
    }
}
