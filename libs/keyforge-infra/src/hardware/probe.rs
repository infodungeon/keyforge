// libs/keyforge-infra/src/hardware/probe.rs
// Low-level hardware probing logic moved from keyforge-compute.
#![allow(unsafe_code)]

use super::CpuCapabilities;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::{CacheType, CpuId};

/// Probes the native host for CPU capabilities.
#[must_use]
pub fn probe_native_host() -> CpuCapabilities {
    let mut caps = CpuCapabilities {
        l1d_size: 32 * 1024,
        l2_size: 256 * 1024,
        l3_size: 8 * 1024 * 1024,
        cache_line_size: 64,
        ..CpuCapabilities::default()
    };

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cpuid = CpuId::new();
        
        // 1. Feature Detection
        if let Some(feat) = cpuid.get_extended_feature_info() {
            caps.has_avx512 = feat.has_avx512f();
        }

        // 2. Cache Topology
        if let Some(cparams) = cpuid.get_cache_parameters() {
            for cache in cparams {
                let size = (cache.associativity() + 1)
                    * (cache.physical_line_partitions() + 1)
                    * (cache.coherency_line_size() + 1)
                    * (cache.sets() + 1);

                match (cache.level(), cache.cache_type()) {
                    (1, CacheType::Data) => {
                        caps.l1d_size = size;
                        caps.cache_line_size = (cache.coherency_line_size() + 1) as _;
                    }
                    (2, _) => caps.l2_size = size,
                    (3, _) => caps.l3_size = size,
                    _ => {}
                }
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        caps.has_neon = true;

        #[cfg(target_os = "macos")]
        if let Some(os_caps) = detect_macos_arm_caps() {
            return os_caps;
        }

        #[cfg(target_os = "windows")]
        if let Some(os_caps) = detect_windows_arm_caps() {
            return os_caps;
        }
    }

    caps
}

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
fn detect_macos_arm_caps() -> Option<CpuCapabilities> {
    use libc::{size_t, sysctlbyname};
    use std::ptr;

    fn get_sysctl_usize(name: &str) -> Option<usize> {
        let mut value: usize = 0;
        let mut size = std::mem::size_of::<usize>() as size_t;
        let c_name = std::ffi::CString::new(name).ok()?;
        unsafe {
            if sysctlbyname(c_name.as_ptr(), &mut value as *mut _ as *mut _, &mut size, ptr::null_mut(), 0) == 0 {
                Some(value)
            } else {
                None
            }
        }
    }

    let mut caps = CpuCapabilities::default();
    caps.has_neon = true;
    caps.l1d_size = get_sysctl_usize("hw.l1dcachesize").unwrap_or(32 * 1024);
    caps.l2_size = get_sysctl_usize("hw.l2cachesize").unwrap_or(256 * 1024);
    caps.l3_size = get_sysctl_usize("hw.l3cachesize").unwrap_or(8 * 1024 * 1024);
    caps.cache_line_size = 64;
    Some(caps)
}

#[cfg(all(target_arch = "aarch64", target_os = "windows"))]
fn detect_windows_arm_caps() -> Option<CpuCapabilities> {
    // ... Windows ARM implementation similar to original compute/hardware.rs but returning CpuCapabilities
    None
}
