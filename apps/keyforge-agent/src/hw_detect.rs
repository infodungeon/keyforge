// apps/keyforge-agent/src/hw_detect.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio;
use tracing::{info, warn};

/// CPU cache and core topology information.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuCacheTopology {
    /// The brand name of the CPU (e.g., "AMD Ryzen 9 5950X").
    pub model: String,
    /// The target architecture (e.g., "`x86_64`", "aarch64").
    pub architecture: String,
    /// The number of logical cores detected.
    pub cores: usize,
    /// L1 Data cache size in kilobytes.
    pub l1_data_kb: Option<u64>, // [Fixed] Changed to u64
    /// L2 cache size in kilobytes.
    pub l2_kb: Option<u64>, // [Fixed] Changed to u64
    /// L3 cache size in kilobytes.
    pub l3_kb: Option<u64>, // [Fixed] Changed to u64
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
        detect_x86_caches(&mut topo);
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
fn detect_x86_caches(topo: &mut CpuCacheTopology) {
    use raw_cpuid::{CacheType, CpuId};
    let cpuid = CpuId::new();
    if let Some(caches) = cpuid.get_cache_parameters() {
        for cache in caches {
            let size_kb_u64 = (cache.sets() as u64
                * cache.associativity() as u64
                * cache.coherency_line_size() as u64)
                / 1024;

            // [Fixed] Direct assignment to u64
            match cache.level() {
                1 => {
                    if cache.cache_type() == CacheType::Data {
                        topo.l1_data_kb = Some(size_kb_u64);
                    }
                }
                2 => topo.l2_kb = Some(size_kb_u64),
                3 => topo.l3_kb = Some(size_kb_u64),
                _ => {}
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn detect_macos_caches(topo: &mut CpuCacheTopology) -> Result<(), AgentError> {
    use libc::{size_t, sysctlbyname};
    use std::ptr;

    fn get_sysctl_u64(name: &str) -> Option<u64> {
        let mut value: u64 = 0;
        let mut size = std::mem::size_of::<u64>() as size_t;
        let c_name = std::ffi::CString::new(name).ok()?;
        // Safety: `sysctlbyname` is a stable macOS system call. We provide a valid C-string name,
        // a pointer to a local `u64` for the value, and a correctly initialized size pointer.
        // The call is synchronous and does not store the pointers beyond the function scope.
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

    // [Fixed] Direct assignment (bytes -> KB)
    if let Some(bytes) = get_sysctl_u64("hw.l1dcachesize") {
        topo.l1_data_kb = Some(bytes / 1024);
    }
    if let Some(bytes) = get_sysctl_u64("hw.l2cachesize") {
        topo.l2_kb = Some(bytes / 1024);
    }
    if let Some(bytes) = get_sysctl_u64("hw.l3cachesize") {
        topo.l3_kb = Some(bytes / 1024);
    }
    Ok(())
}

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
fn detect_windows_arm_caches(topo: &mut CpuCacheTopology) -> Result<(), AgentError> {
    use std::alloc::{alloc, Layout};
    use std::ptr;
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationCache, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let mut len: u32 = 0;
    // Safety: Initial call with null pointer to retrieve the required buffer size.
    // This is standard Windows API pattern for variable-length result structures.
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

    // Safety: Buffer is allocated with sufficient size and alignment for the requested struct.
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return Ok(());
    }

    // Safety: `GetLogicalProcessorInformationEx` is called with a verified valid buffer pointer
    // and the correct length retrieved from the previous call. We manually advance the pointer
    // using the `Size` field of each structure to ensure we remain within valid memory bounds.
    unsafe {
        if GetLogicalProcessorInformationEx(RelationCache, ptr as *mut _, &mut len) != 0 {
            let mut offset = 0;
            while offset < len {
                let info =
                    &*(ptr.add(offset as usize) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX);
                let cache = &info.u.Cache;
                let size_kb_u64 = cache.Size as u64 / 1024;

                // [Fixed] Direct assignment
                match cache.Level {
                    1 => topo.l1_data_kb = Some(size_kb_u64),
                    2 => topo.l2_kb = Some(size_kb_u64),
                    3 => topo.l3_kb = Some(size_kb_u64),
                    _ => {}
                }
                offset += info.Size;
            }
        }
        std::alloc::dealloc(ptr, layout);
    }
    Ok(())
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_topology_detection() {
        let topo = detect_topology().await.unwrap();
        assert!(!topo.model.is_empty());
        assert!(topo.cores >= 1);
    }
}
