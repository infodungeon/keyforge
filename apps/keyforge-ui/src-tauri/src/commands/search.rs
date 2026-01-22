use crate::error::CommandError;
use crate::models::{JobStatusUpdate, RegisterJobRequest, SearchUpdate, StartSearchRequest};
use crate::state::{LocalWorkerState, SearchState, SessionState};
use crate::utils::get_data_dir;
use keyforge_evolution::{OptimizationControl, ProgressCallback};
use keyforge_infra::{AssetLoader, HiveClient};
use keyforge_model::KeyCode;
use keyforge_protocol::{JobRequest, JobResponse, JobStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Window};
use tauri_plugin_shell::ShellExt;

/// Dispatches a job to the Hive API.
///
/// # Errors
///
/// Returns `CommandError` if the network request fails or the configuration is invalid.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn cmd_dispatch_job(
    _state: tauri::State<'_, SessionState>,
    hive_url: String,
    hive_secret: String,
    request: RegisterJobRequest,
) -> Result<String, CommandError> {
    use keyforge_infra::net::client::ClientConfig;
    // Search commands interact with API, asset_url doesn't matter much here but required by struct
    let asset_url = hive_url.replace("3000", "3001");
    let config = ClientConfig {
        api_url: hive_url,
        asset_url,
        secret: Some(hive_secret),
        ..Default::default()
    };
    let client = HiveClient::new(config).map_err(|e| CommandError::Config(e.to_string()))?;

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

/// Polls the server for the current status of a job.
///
/// # Errors
///
/// Returns `CommandError` if the network request fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn cmd_poll_hive_status(
    hive_url: String,
    hive_secret: String,
    job_id: String,
) -> Result<JobStatusUpdate, CommandError> {
    use keyforge_infra::net::client::ClientConfig;
    let asset_url = hive_url.replace("3000", "3001");
    let config = ClientConfig {
        api_url: hive_url,
        asset_url,
        secret: Some(hive_secret),
        ..Default::default()
    };
    let client = HiveClient::new(config).map_err(|e| CommandError::Config(e.to_string()))?;

    let path = format!("jobs/{job_id}/status");
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

/// Starts or stops the local background worker process.
///
/// # Errors
///
/// Returns `CommandError` if the agent process fails to spawn or terminate.
///
/// # Panics
///
/// Panics if the worker child lock is poisoned.
#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    clippy::unwrap_used,
    clippy::missing_panics_doc
)]
pub fn cmd_toggle_local_worker(
    app: AppHandle,
    state: tauri::State<'_, LocalWorkerState>,
    enabled: bool,
    hive_url: String,
    _hive_secret: String,
) -> Result<String, CommandError> {
    let mut child_guard = state.child.lock().unwrap();

    if enabled {
        if child_guard.is_some() {
            return Ok("Worker already running".into());
        }

        let data_dir = get_data_dir(&app).map_err(CommandError::Internal)?;

        // Spawn sidecar
        let (mut _rx, child) = app
            .shell()
            .sidecar("keyforge-agent")
            .map_err(|e| CommandError::Internal(format!("Failed to find sidecar: {e}")))?
            .args([
                "worker",
                "--hive",
                &hive_url,
                "--data-dir",
                &data_dir.to_string_lossy(),
            ])
            .spawn()
            .map_err(|e| CommandError::Internal(format!("Failed to spawn worker: {e}")))?;

        *child_guard = Some(child);
        Ok("Worker started".into())
    } else if let Some(child) = child_guard.take() {
        child
            .kill()
            .map_err(|e| CommandError::Internal(format!("Failed to kill worker: {e}")))?;
        Ok("Worker stopped".into())
    } else {
        Ok("Worker not running".into())
    }
}

struct TauriProgressCallback {
    window: Window,
    stop_flag: Arc<AtomicBool>,
    last_emit: std::sync::Mutex<std::time::Instant>,
    registry: Arc<keyforge_model::KeycodeRegistry>,
}

impl ProgressCallback for TauriProgressCallback {
    fn on_progress(
        &self,
        epoch: usize,
        score: f32,
        layout: &[KeyCode],
        ips: f32,
    ) -> OptimizationControl {
        if self.stop_flag.load(Ordering::SeqCst) {
            return OptimizationControl::Stop;
        }

        #[allow(clippy::unwrap_used)]
        let mut last = self.last_emit.lock().unwrap();
        if last.elapsed().as_millis() > 100 {
            *last = std::time::Instant::now();

            // Convert layout codes to string representation
            let mut layout_str = String::new();
            for code in layout {
                let label = self.registry.get_label(*code);
                layout_str.push_str(&label);
                layout_str.push(' ');
            }

            let update = SearchUpdate {
                epoch,
                score,
                layout: layout_str.trim().to_string(),
                ips,
            };
            #[allow(clippy::unwrap_used)]
            let _ = self.window.emit("search_update", update);
        }
        OptimizationControl::Continue
    }
}

/// Starts a local optimization search.
///
/// # Errors
///
/// Returns `CommandError` if the session preparation fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn cmd_start_search(
    window: Window,
    state: tauri::State<'_, SessionState>,
    search_state: tauri::State<'_, SearchState>,
    request: StartSearchRequest,
) -> Result<String, CommandError> {
    // 1. Get Active Job Context
    let job = {
        let job_guard = state.active_job.read().await;
        job_guard
            .as_ref()
            .ok_or_else(|| CommandError::Internal("No active job found".into()))?
            .clone()
    };

    // 2. Prepare Engine Request via Runner
    let options = keyforge_runner::RunnerOptions {
        timeout_sec: request.search_params.get_search_steps() as u64 / 100, // Dummy heuristic for now
        seed: request.search_params.seed,
        keycodes_file: "default".to_string(),
        ..Default::default()
    };

    let session =
        keyforge_runner::OptimizationRunner::prepare_session(state.assets.as_ref(), &job, &options)
            .await
            .map_err(|e| CommandError::Internal(e.to_string()))?;

    // Reset stop flag
    search_state.stop_flag.store(false, Ordering::SeqCst);

    // 3. Spawn Optimization Task via Runner
    let stop_flag = search_state.stop_flag.clone();
    let window_handle = window.clone();

    // Resolve keycodes for labeling in the callback
    let keycodes = state
        .assets
        .load::<keyforge_model::KeycodeRegistry>("default")
        .await
        .unwrap_or_else(|_| Arc::new(keyforge_model::KeycodeRegistry::new_with_defaults()));

    let callback = TauriProgressCallback {
        window: window_handle.clone(),
        stop_flag: stop_flag.clone(),
        last_emit: std::sync::Mutex::new(std::time::Instant::now()),
        registry: keycodes,
    };

    tokio::spawn(async move {
        match keyforge_runner::OptimizationRunner::run(
            session,
            job.id().unwrap_or_else(|_| "ui-job".to_string()),
            stop_flag,
            callback,
            options,
            &job,
        )
        .await
        {
            Ok(result) => {
                let _ = window_handle.emit("search_finished", result);
            }
            Err(e) => {
                let _ = window_handle.emit("search_error", e.to_string());
            }
        }
    });

    Ok("Search started".into())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_stop_search(search_state: tauri::State<'_, SearchState>) {
    search_state.stop_flag.store(true, Ordering::SeqCst);
}
