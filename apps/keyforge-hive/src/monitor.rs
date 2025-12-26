use std::sync::Arc;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::sync::Mutex;

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    pub fn get_memory_used(&self) -> u64 {
        self.sys.used_memory()
    }

    pub fn get_cpu_usage(&self) -> f32 {
        self.sys.global_cpu_info().cpu_usage()
    }

    pub fn get_uptime(&self) -> u64 {
        System::uptime()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedMonitor = Arc<Mutex<SystemMonitor>>;
