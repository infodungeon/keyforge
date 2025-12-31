use crate::error::CommandError;
use keyforge_infra::HiveClient;
use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tauri::AppHandle;

#[derive(Serialize)]
pub struct SystemHealth {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime: u64,
    pub cores: usize,
}

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
