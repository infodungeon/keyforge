use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio;
use tracing::{info, warn};

/// CPU cache and core topology information.
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

use crate::agent::errors::AgentError;

/// Detects the CPU topology and cache sizes of the host machine.
///
/// This information is used to tune the optimization process (e.g., fitting data structures in L2).
///
/// # Errors
/// Returns `AgentError::Hardware` if cache information cannot be retrieved on supported platforms.
pub async fn detect_topology() -> Result<CpuCacheTopology, AgentError> {
    let mut topo = CpuCacheTopology::default();

    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    sys.refresh_all();

    if let Some(cpu) = sys.cpus().first() {
        topo.model = cpu.brand().trim().to_string();
    }
    topo.cores = sys.cpus().len();

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        detect_x86_caches(&mut topo)?;
    }

    #[cfg(target_os = "macos")]
    {
        detect_macos_caches(&mut topo)?;
    }

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        detect_windows_arm_caches(&mut topo)?;
    }

    // Task 27: Structured logging
    info!(
        model = %topo.model,
        cores = topo.cores,
        "hardware detected"
    );
    if let Some(l2) = topo.l2_kb {
        info!(l2_kb = l2, "L2 cache detected, tuning for trigram table");
    } else {
        warn!("L2 cache size unknown, using safe defaults for sizing");
    }

    Ok(topo)
}

/// Returns a list of SIMD features the binary was compiled for.
#[must_use]
pub fn get_compile_features() -> &'static [&'static str] {
    if cfg!(target_feature = "avx512f") {
        &["AVX512"]
    } else if cfg!(target_feature = "avx2") {
        &["AVX2"]
    } else if cfg!(target_feature = "avx") {
        &["AVX"]
    } else if cfg!(target_feature = "sse4.2") {
        &["SSE4.2"]
    } else if cfg!(target_feature = "sse2") {
        &["SSE2"]
    } else if cfg!(target_feature = "neon") {
        &["NEON"]
    } else {
        &["SCALAR (No SIMD)"]
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn detect_x86_caches(topo: &mut CpuCacheTopology) -> Result<(), AgentError> {
    use raw_cpuid::{CacheType, CpuId};
    let cpuid = CpuId::new();
    if let Some(caches) = cpuid.get_cache_parameters() {
        for cache in caches {
            let size_kb_u64 = (cache.sets() as u64
                * cache.associativity() as u64
                * cache.coherency_line_size() as u64)
                / 1024;
            let size_kb = u64_to_usize_saturating(size_kb_u64);

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
    Ok(())
}

#[cfg(target_os = "macos")]
fn detect_macos_caches(topo: &mut CpuCacheTopology) -> Result<(), AgentError> {
    // Task 52: Avoid sysctl binary spawn, use libc
    use libc::{size_t, sysctlbyname};
    use std::ptr;

    fn get_sysctl_u64(name: &str) -> Option<u64> {
        let mut value: u64 = 0;
        let mut size = std::mem::size_of::<u64>() as size_t;
        let c_name = std::ffi::CString::new(name).ok()?;
        // SAFETY: sysctlbyname is a safe call with valid pointers
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

    if let Some(bytes) = get_sysctl_u64("hw.l1dcachesize") {
        topo.l1_data_kb = Some(u64_to_usize_saturating(bytes / 1024));
    }
    if let Some(bytes) = get_sysctl_u64("hw.l2cachesize") {
        topo.l2_kb = Some(u64_to_usize_saturating(bytes / 1024));
    }
    if let Some(bytes) = get_sysctl_u64("hw.l3cachesize") {
        topo.l3_kb = Some(u64_to_usize_saturating(bytes / 1024));
    }
    Ok(())
}

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
fn detect_windows_arm_caches(topo: &mut CpuCacheTopology) -> Result<(), AgentError> {
    // Task 51: ARM-Windows cache detection
    use std::alloc::{alloc, Layout};
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationCache, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let mut len: u32 = 0;
    // SAFETY: First call to get required buffer size
    unsafe {
        GetLogicalProcessorInformationEx(RelationCache, ptr::null_mut(), &mut len);
    }
    if len == 0 {
        return Ok(());
    }

    let layout = Layout::from_size_align(
        len as usize,
        std::mem::align_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>(),
    )
    .map_err(|e| AgentError::Hardware(format!("Memory layout allocation failed: {}", e)))?;
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return Ok(());
    }

    // SAFETY: Buffer allocated with correct size and alignment
    unsafe {
        if GetLogicalProcessorInformationEx(RelationCache, ptr as *mut _, &mut len) != 0 {
            let mut offset = 0;
            while offset < len {
                let info =
                    &*(ptr.add(offset as usize) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX);
                // RELATION_CACHE_INFORMATION is at the end of the union
                let cache = &info.u.Cache;
                let size_kb_u64 = (cache.Size as u64 / 1024);
                let size_kb = u64_to_usize_saturating(size_kb_u64);

                match cache.Level {
                    1 => topo.l1_data_kb = Some(size_kb),
                    2 => topo.l2_kb = Some(size_kb),
                    3 => topo.l3_kb = Some(size_kb),
                    _ => {}
                }
                offset += info.Size;
            }
        }
        std::alloc::dealloc(ptr, layout);
    }
    Ok(())
}

/// Safely casts a u64 to usize, saturating at usize::MAX and warning if truncation occurs.
/// This prevents integer overflow issues on 32-bit systems when handling large cache sizes.
fn u64_to_usize_saturating(value: u64) -> usize {
    if value > usize::MAX as u64 {
        warn!(
            "Value {} exceeds platform usize limit {}, saturating to limit.",
            value,
            usize::MAX
        );
        usize::MAX
    } else {
        value as usize
    }
}
