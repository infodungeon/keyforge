use crate::error::CommandError;
use crate::models::{JobStatusUpdate, RegisterJobRequest, StartSearchRequest};
use crate::state::{LocalWorkerState, SearchState, SessionState};
use keyforge_infra::HiveClient;
use keyforge_protocol::{JobRequest, JobResponse, JobStatus};
use tauri::{AppHandle, Window};

/// Dispatches a new search job to the remote Hive server.
#[tauri::command]
pub async fn cmd_dispatch_job(
    _state: tauri::State<'_, SessionState>,
    hive_url: String,
    hive_secret: String,
    request: RegisterJobRequest,
) -> Result<String, CommandError> {
    use keyforge_infra::net::client::ClientConfig;
    let config = ClientConfig {
        base_url: hive_url,
        secret: Some(hive_secret),
        ..Default::default()
    };
    let client = HiveClient::new(config)
        .map_err(|e| CommandError::Config(e.to_string()))?;

    // Convert UI model to Protocol model (they are aliases but explicit cast helps clarity)
    let job_req: JobRequest = request;

    let resp = client
        .post("jobs")
        .json(&job_req)
        .send()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(CommandError::Network(format!(
            "Server Error: {}",
            resp.status()
        )));
    }

    let body: JobResponse = resp
        .json()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    Ok(body.job_id)
}

/// Polls the Hive server for the current status of a remote search job.
#[tauri::command]
pub async fn cmd_poll_hive_status(
    hive_url: String,
    hive_secret: String,
    job_id: String,
) -> Result<JobStatusUpdate, CommandError> {
    use keyforge_infra::net::client::ClientConfig;
    let config = ClientConfig {
        base_url: hive_url,
        secret: Some(hive_secret),
        ..Default::default()
    };
    let client = HiveClient::new(config)
        .map_err(|e| CommandError::Config(e.to_string()))?;

    let path = format!("jobs/{}/status", job_id);
    let resp = client
        .get(&path)
        .send()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(CommandError::Network(format!(
            "Server Error: {}",
            resp.status()
        )));
    }

    let status: JobStatus = resp
        .json()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    Ok(JobStatusUpdate {
        active_nodes: status.active_nodes,
        best_score: status.best_score.unwrap_or(0.0),
        best_layout: status.best_layout.unwrap_or_default(),
    })
}

#[tauri::command]
/// Toggles the local compute worker for distributed job processing.
pub fn cmd_toggle_local_worker(
    _app: AppHandle,
    _state: tauri::State<'_, LocalWorkerState>,
    _enabled: bool,
    _hive_url: String,
    _hive_secret: String,
) -> Result<String, CommandError> {
    // TODO: Implement local worker spawning logic using keyforge-agent binary
    Ok("Toggled".into())
}

#[tauri::command]
/// Initiates a local search/optimization operation.
pub async fn cmd_start_search(
    _window: Window,
    _state: tauri::State<'_, SessionState>,
    _search_state: tauri::State<'_, SearchState>,
    _request: StartSearchRequest,
) -> Result<String, CommandError> {
    Err(CommandError::Internal(
        "Local optimization temporarily disabled during refactor".into(),
    ))
}

/// Signals a stop to the currently running local search operation.
#[tauri::command]
pub fn cmd_stop_search(search_state: tauri::State<SearchState>) {
    if let Ok(mut flag) = search_state.stop_flag.lock() {
        *flag = true;
    }
}
