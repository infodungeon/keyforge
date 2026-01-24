use keyforge_physics::{ArmNeonConfig, IntelEngineConfig};
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::{CacheType, CpuId};
use std::env;

#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub vendor: String,
    pub architecture: String,
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

impl From<CpuTopology> for ArmNeonConfig {
    fn from(topo: CpuTopology) -> Self {
        Self {
            l1d_size_bytes: topo.l1d_size_bytes,
            l2_size_bytes: topo.l2_size_bytes,
        }
    }
}

impl Default for CpuTopology {
    fn default() -> Self {
        Self {
            vendor: "Unknown".to_string(),
            architecture: env::consts::ARCH.to_string(),
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
        let mut topology = CpuTopology::default();

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let cpuid = CpuId::new();
            if let Some(vendor) = cpuid.get_vendor_info() {
                topology.vendor = vendor.as_str().to_string();
            }

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
        }

        #[cfg(target_arch = "aarch64")]
        {
            topology.vendor = "ARM".to_string();

            #[cfg(target_os = "macos")]
            {
                if let Some(topo) = detect_macos_arm_topology() {
                    topology = topo;
                }
            }

            #[cfg(target_os = "windows")]
            {
                if let Some(topo) = detect_windows_arm_topology() {
                    topology = topo;
                }
            }
        }

        // Linux fallback for cache detection (works for x86 and ARM)
        #[cfg(target_os = "linux")]
        {
            // If we don't have cache info yet, or we're on ARM, try /sys
            if topology.l1d_size_bytes == 32 * 1024 || cfg!(target_arch = "aarch64") {
                if let Some(linux_topo) = detect_linux_topology() {
                    // Only override if we got actual data
                    if linux_topo.l1d_size_bytes != 32 * 1024 {
                        topology.l1d_size_bytes = linux_topo.l1d_size_bytes;
                    }
                    if linux_topo.l2_size_bytes != 256 * 1024 {
                        topology.l2_size_bytes = linux_topo.l2_size_bytes;
                    }
                    if linux_topo.l3_size_bytes != 8 * 1024 * 1024 {
                        topology.l3_size_bytes = linux_topo.l3_size_bytes;
                    }
                    if linux_topo.vendor != "Unknown" {
                        topology.vendor = linux_topo.vendor;
                    }
                }
            }
        }

        topology
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_topology() -> Option<CpuTopology> {
    let mut topo = CpuTopology::default();

    // Try to get vendor/model from /proc/cpuinfo
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("vendor_id")
                || line.starts_with("Hardware")
                || line.starts_with("Model")
            {
                if let Some(v) = line.split(':').nth(1) {
                    topo.vendor = v.trim().to_string();
                    break;
                }
            }
        }
    }

    // Try to read cache sizes from /sys/devices/system/cpu/cpu0/cache/
    for i in 0..4 {
        let path = format!("/sys/devices/system/cpu/cpu0/cache/index{i}/");
        let level_path = format!("{path}level");
        let type_path = format!("{path}type");
        let size_path = format!("{path}size");

        let Ok(level_str) = std::fs::read_to_string(level_path) else {
            continue;
        };
        let level = level_str.trim().parse::<u8>().ok()?;

        let Ok(ctype) = std::fs::read_to_string(type_path) else {
            continue;
        };
        let ctype = ctype.trim().to_lowercase();

        let Ok(size_str) = std::fs::read_to_string(size_path) else {
            continue;
        };
        let size_str = size_str.trim();

        let size_bytes = if let Some(stripped) = size_str.strip_suffix('K') {
            stripped.parse::<usize>().ok()? * 1024
        } else if let Some(stripped) = size_str.strip_suffix('M') {
            stripped.parse::<usize>().ok()? * 1024 * 1024
        } else {
            size_str.parse::<usize>().ok()?
        };

        match (level, ctype.as_str()) {
            (1, "data") => topo.l1d_size_bytes = size_bytes,
            (2, _) => topo.l2_size_bytes = size_bytes,
            (3, _) => topo.l3_size_bytes = size_bytes,
            _ => {}
        }
    }

    Some(topo)
}

#[cfg(target_os = "macos")]
fn detect_macos_arm_topology() -> Option<CpuTopology> {
    use libc::{size_t, sysctlbyname};
    use std::ptr;

    fn get_sysctl_usize(name: &str) -> Option<usize> {
        let mut value: usize = 0;
        let mut size = std::mem::size_of::<usize>() as size_t;
        let c_name = std::ffi::CString::new(name).ok()?;
        unsafe {
            if sysctlbyname(
                c_name.as_ptr(),
                &mut value as *mut _ as *mut _,
                &mut size,
                ptr::null_mut(),
                0,
            ) == 0
            {
                Some(value)
            } else {
                None
            }
        }
    }

    let mut topo = CpuTopology::default();
    topo.vendor = "Apple".to_string();
    if let Some(val) = get_sysctl_usize("hw.l1dcachesize") {
        topo.l1d_size_bytes = val;
    }
    if let Some(val) = get_sysctl_usize("hw.l2cachesize") {
        topo.l2_size_bytes = val;
    }
    if let Some(val) = get_sysctl_usize("hw.l3cachesize") {
        topo.l3_size_bytes = val;
    }

    Some(topo)
}

#[cfg(target_os = "windows")]
fn detect_windows_arm_topology() -> Option<CpuTopology> {
    use std::alloc::{alloc, Layout};
    use std::ptr;
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationCache, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let mut len: u32 = 0;
    unsafe {
        GetLogicalProcessorInformationEx(RelationCache, ptr::null_mut(), &mut len);
    }
    if len == 0 {
        return None;
    }

    let layout = Layout::from_size_align(
        len as usize,
        std::mem::align_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>(),
    )
    .ok()?;

    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return None;
    }

    let mut topo = CpuTopology::default();
    topo.vendor = "Unknown (Windows ARM)".to_string();

    unsafe {
        if GetLogicalProcessorInformationEx(RelationCache, ptr as *mut _, &mut len) != 0 {
            let mut offset = 0;
            while offset < len {
                let info =
                    &*(ptr.add(offset as usize) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX);
                let cache = &info.u.Cache;
                let size_bytes = cache.Size as usize;

                match cache.Level {
                    1 => topo.l1d_size_bytes = size_bytes,
                    2 => topo.l2_size_bytes = size_bytes,
                    3 => topo.l3_size_bytes = size_bytes,
                    _ => {}
                }
                offset += info.Size;
            }
        }
        std::alloc::dealloc(ptr, layout);
    }

    Some(topo)
}

#[keyforge_testing_macros::kf_test]
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
