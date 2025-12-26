use tracing::{info, warn};

/// Configures the process to run with background priority.
///
/// This reduces impact on the host system's interactive responsiveness.
/// # Platform-specific behavior:
/// - **Unix**: Sets nice value to 10.
/// - **Windows**: Sets priority class to `IDLE_PRIORITY_CLASS`.
pub fn set_background_priority() {
    #[cfg(unix)]
    {
        // SAFETY: Calling setpriority on current process (0, 0) is safe and common for nicing.
        unsafe {
            // Nice value 10 (lower priority)
            libc::setpriority(0, 0, 10);
        }
        info!(priority = 10, "process priority lowered");
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, SetPriorityClass, IDLE_PRIORITY_CLASS,
        };

        // SAFETY: GetCurrentProcess returns a pseudo-handle for the current process,
        // and SetPriorityClass is safe when passed a valid handle.
        unsafe {
            let handle = GetCurrentProcess();
            SetPriorityClass(handle, IDLE_PRIORITY_CLASS);
        }
        info!(class = "IDLE", "process priority lowered");
    }
}

/// Configures the Rayon global thread pool with affinities and resource limits.
///
/// # Parameters
/// - `is_background`: If true, leaves some cores free for the system.
///
/// # Security & Stability
/// - Sets an explicit stack size of 2MB to prevent overflow.
/// - Attempts to pin threads to physical cores to improve cache locality.
pub fn configure_global_thread_pool(is_background: bool) {
    // 1. Determine Target Thread Count
    let max_threads = num_cpus::get();
    let threads = if is_background {
        // Leave 2 cores free for system/UI if possible, min 1
        max_threads.saturating_sub(2).max(1)
    } else {
        // Use all cores
        max_threads
    };

    // 2. Retrieve Core IDs for Affinity
    let core_ids = core_affinity::get_core_ids().unwrap_or_default();
    if core_ids.is_empty() {
        warn!("could not detect core IDs, affinity pinning disabled");
    } else {
        info!(count = core_ids.len(), "detected physical/logical cores");
    }

    // 3. Configure Rayon Global Pool
    let res = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        // Task 35: Add explicit stack-size limit (2MB)
        .stack_size(2 * 1024 * 1024)
        .start_handler(move |thread_idx| {
            // Pin thread to core if possible
            if !core_ids.is_empty() {
                // Simple round-robin assignment
                let core_idx = thread_idx % core_ids.len();
                let core_id = core_ids[core_idx];
                if !core_affinity::set_for_current(core_id) {
                    warn!(
                        thread_idx = thread_idx,
                        core_id = ?core_id,
                        "failed to pin worker"
                    );
                }
            }
        })
        .build_global();

    match res {
        Ok(_) => info!(
            threads = threads,
            total_cores = max_threads,
            "thread pool configured"
        ),
        Err(e) => warn!(
            error = %e,
            "thread pool already initialized, skipping config"
        ),
    }
}
