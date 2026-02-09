use crate::error::CommandError;
use crate::models::{JobStatusUpdate, RegisterJobRequest, SearchUpdate, StartSearchRequest};
use crate::state::{LocalWorkerState, SearchState, SessionState};
use crate::utils::get_data_dir;
use keyforge_adapter::loader::AssetLoader;
use keyforge_compute::Runtime;
use keyforge_evolution::{OptimizationControl, ProgressCallback};
use keyforge_infra::HiveClient;
use keyforge_model::{KeyCode, KeyboardDefinition};
use keyforge_protocol::{JobRequest, JobResponse, JobStatusDto};
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
    let client = HiveClient::new(config)?;

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
    let client = HiveClient::new(config)?;

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

    let status: JobStatusDto = resp
        .json()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    Ok(JobStatusUpdate {
        active_nodes: match &status {
            JobStatusDto::Running { active_nodes, .. } => *active_nodes,
            _ => 0,
        },
        best_score: match &status {
            JobStatusDto::Running { current_best, .. } => current_best.as_ref().map_or(0.0, |s| {
                // SAFETY: TYPE-001 Exception: DTO conversion.
                #[allow(clippy::cast_precision_loss)]
                let val = s.raw() as f32;
                val / 1_000_000.0
            }),
            JobStatusDto::Completed { final_score, .. } => {
                // SAFETY: TYPE-001 Exception: DTO conversion.
                #[allow(clippy::cast_precision_loss)]
                let val = final_score.raw() as f32;
                val / 1_000_000.0
            }
            JobStatusDto::Pending => 0.0,
        },
        best_layout: match &status {
            JobStatusDto::Completed { final_layout, .. } => {
                use std::fmt::Write;
                let mut s = String::new();
                for code in &final_layout.keys {
                    // SAFETY: ARCH-006 Exception: Serialized DTO field access.
                    let _ = write!(s, "{} ", code.raw());
                }
                s.trim().to_string()
            }
            _ => String::new(),
        },
    })
}

/// Starts or stops the local background worker process.
///
/// # Errors
///
/// Returns `CommandError` if the agent process fails to spawn or terminate.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
pub fn cmd_toggle_local_worker(
    app: AppHandle,
    state: tauri::State<'_, LocalWorkerState>,
    enabled: bool,
    hive_url: String,
    _hive_secret: String,
) -> Result<String, CommandError> {
    let mut child_guard = state
        .child
        .lock()
        .map_err(|_| CommandError::Internal("Mutex poisoned".into()))?;

    if enabled {
        if child_guard.is_some() {
            return Ok("Worker already running".into());
        }

        let data_dir = get_data_dir(&app)?;

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
        score: keyforge_model::Score,
        layout: &[KeyCode],
        ips: f32,
    ) -> OptimizationControl {
        if self.stop_flag.load(Ordering::SeqCst) {
            return OptimizationControl::Stop;
        }

        // SAFETY: TYPE-003 Exception: Callback rate limiting. Poisoned lock is unrecoverable here.
        let Ok(mut last) = self.last_emit.lock() else {
            return OptimizationControl::Abort;
        };
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
                score: score.to_f32(),
                layout: layout_str.trim().to_string(),
                ips,
            };
            // SAFETY: TYPE-003 Exception: UI event emission.
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

    // 2. Prepare Engine Request via SessionBuilder
    let builder = keyforge_compute::SessionBuilder::new(state.assets.as_ref())
        .with_keyboard_def(std::sync::Arc::new(KeyboardDefinition::from_geometry(
            job.to_domain_geometry(),
            "local",
        )))
        .with_corpus(&job.to_domain_corpus_sources())
        .await?
        .with_cost_matrix(&job.to_domain_cost_matrix())
        .await?
        .with_keycodes("default")
        .await?
        .with_rubric(keyforge_adapter::conversion::to_domain_rubric(
            &job.to_domain_weights()?,
        ))
        .with_config(keyforge_model::SearchConfig::Annealing {
            steps: request.search_params.get_search_steps(),
            start_temp: request.search_params.get_temp_max(),
            end_temp: request.search_params.get_temp_min(),
            seed: request.search_params.seed.unwrap_or(42),
            patience: request.search_params.get_search_patience(),
            reheats: request.search_params.get_reheats(),
            reheat_factor: request.search_params.get_reheat_factor(),
            include_thumbs: request.search_params.include_thumbs,
        });

    let session = builder.build()?;

    // Reset stop flag
    search_state.stop_flag.store(false, Ordering::SeqCst);

    // 3. Spawn Optimization Task via session
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
        let runtime = Runtime::from(session);
        match runtime
            .run_optimization(callback, &job.to_domain_pinned_keys())
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
