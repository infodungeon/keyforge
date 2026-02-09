// apps/keyforge-agent/src/agent/hardware.rs

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::CpuId;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub cpu_model: String,
    pub cores: i32,
    pub l2_cache_kb: Option<i32>,
    pub capabilities: Vec<String>,
}

impl HardwareInfo {
    #[must_use]
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();

        let cpu_model = sys
            .cpus()
            .first()
            .map_or_else(|| "Unknown CPU".to_string(), |cpu| cpu.brand().to_string());

        let cores = sys.cpus().len();
        let cores_i32: i32 = cores.try_into().unwrap_or_default();

        let l2_cache_kb = detect_l2_cache_kb();
        let capabilities = detect_capabilities();

        Self {
            cpu_model,
            cores: cores_i32,
            l2_cache_kb,
            capabilities,
        }
    }
}

fn detect_capabilities() -> Vec<String> {
    let mut caps = Vec::new();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cpuid = CpuId::new();
        if let Some(feature_info) = cpuid.get_feature_info() {
            if feature_info.has_sse() {
                caps.push("sse".into());
            }
            if feature_info.has_sse2() {
                caps.push("sse2".into());
            }
            if feature_info.has_avx() {
                caps.push("avx".into());
            }
        }
        if let Some(extended_features) = cpuid.get_extended_feature_info() {
            if extended_features.has_avx2() {
                caps.push("avx2".into());
            }
            if extended_features.has_avx512f() {
                caps.push("avx512".into());
            }
        }
    }
    caps
}

fn detect_l2_cache_kb() -> Option<i32> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cpuid = CpuId::new();
        if cpuid.get_l1_cache_and_tlb_info().is_some() {
            if let Some(params) = cpuid.get_cache_parameters() {
                for cache in params {
                    if cache.level() == 2 {
                        let size_bytes = (cache.associativity() + 1)
                            * (cache.physical_line_partitions() + 1)
                            * (cache.coherency_line_size() + 1)
                            * (cache.sets() + 1);
                        return Some((size_bytes / 1024).try_into().unwrap_or_default());
                    }
                }
            }
        }
    }

    // Fallback for non-x86 or if cpuid failed
    // On Linux we could read /sys/devices/system/cpu/cpu0/cache/index2/size
    #[cfg(target_os = "linux")]
    {
        // SAFETY: ARCH-005 Exception: Hardware detection requires direct filesystem access
        // to system sysfs nodes. This is platform-specific initialization code,
        // not part of a pure physics or evolution kernel.
        let path_str = "/sys/devices/system/cpu/cpu0/cache/index2/size";
        let safe_sys_path = keyforge_model::types::path::SafePath::from_trusted_root_path(
            std::path::PathBuf::from(path_str),
        );
        if let Ok(content) = keyforge_infra::fs::io::read_to_string_limited(&safe_sys_path, 1024) {
            let s = content.trim();
            if let Some(stripped) = s.strip_suffix('K') {
                if let Ok(kb) = stripped.parse::<i32>() {
                    return Some(kb);
                }
            } else if let Some(stripped) = s.strip_suffix('M') {
                if let Ok(mb) = stripped.parse::<i32>() {
                    return Some(mb * 1024);
                }
            }
        }
    }

    None
}
