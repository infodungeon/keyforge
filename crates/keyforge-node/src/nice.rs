use tracing::info;

pub fn set_background_priority() {
    #[cfg(unix)]
    {
        // Nice values range from -20 (high) to 19 (low). 
        // 10 is a good "background" value.
        unsafe {
            // 0 = PRIO_PROCESS, 0 = Current Process ID, 10 = Nice Level
            libc::setpriority(0, 0, 10);
        }
        info!("📉 Process priority lowered (Unix Nice: 10)");
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, SetPriorityClass, IDLE_PRIORITY_CLASS,
        };

        unsafe {
            // IDLE_PRIORITY_CLASS ensures it only runs when the system is otherwise idle
            let handle = GetCurrentProcess();
            SetPriorityClass(handle, IDLE_PRIORITY_CLASS);
        }
        info!("📉 Process priority lowered (Windows Idle Class)");
    }
}

pub fn configure_global_thread_pool(is_background: bool) {
    let max_threads = num_cpus::get();
    
    // If background, leave 2 cores free (or 1 if dual core), otherwise use all.
    // This ensures the UI remains snappy even while optimization runs.
    let threads = if is_background {
        max_threads.saturating_sub(2).max(1)
    } else {
        max_threads
    };

    // Configure Rayon (Parallel Iterator) Global Pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("Failed to configure Rayon thread pool");

    info!(
        "🧵 Thread Pool configured: {} worker threads (System total: {})",
        threads, max_threads
    );
}