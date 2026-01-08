use crate::error::CommandError;
use keyforge_infra::HiveClient;
use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tauri::AppHandle;

/// Represents the current physical health metrics of the local system.
#[derive(Serialize)]
pub struct SystemHealth {
    /// Global CPU usage percentage.
    pub cpu_usage: f32,
    /// Amount of physical memory used in bytes.
    pub memory_used: u64,
    /// Total amount of physical memory in bytes.
    pub memory_total: u64,
    /// System uptime in seconds.
    pub uptime: u64,
    /// Number of logical CPU cores.
    pub cores: usize,
}

/// Retrieves the current system health metrics.
#[tauri::command]
pub fn cmd_get_system_health(_app: AppHandle) -> SystemHealth {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    sys.refresh_all();

    SystemHealth {
        cpu_usage: sys.global_cpu_usage(),
        memory_used: sys.used_memory(),
        memory_total: sys.total_memory(),
        uptime: System::uptime(),
        cores: sys.cpus().len(),
    }
}

/// Checks the health of a remote Hive server.
#[tauri::command]
pub async fn cmd_check_hive_health(hive_url: String) -> Result<String, CommandError> {
    let client =
        HiveClient::new(hive_url, None).map_err(|e| CommandError::Config(e.to_string()))?;

    let resp = client
        .get("health")
        .send()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    if resp.status().is_success() {
        Ok("OK".to_string())
    } else {
        Err(CommandError::Network(format!(
            "Health check failed: {}",
            resp.status()
        )))
    }
}
