use crate::error::CommandError;
use crate::models::{JobStatusUpdate, RegisterJobRequest, StartSearchRequest, SearchUpdate};
use crate::state::{LocalWorkerState, SearchState, SessionState};
use crate::utils::get_data_dir;
use keyforge_infra::HiveClient;
use keyforge_protocol::{JobRequest, JobResponse, JobStatus};
use tauri::{AppHandle, Emitter, Window};
use tauri_plugin_shell::ShellExt;
use keyforge_evolution::{ProgressCallback, optimize_with_callback};
use keyforge_physics::EngineRequest;
use keyforge_model::types::KeyCode;
use std::sync::{Arc, Mutex};

#[tauri::command]
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
    let client = HiveClient::new(config)
        .map_err(|e| CommandError::Config(e.to_string()))?;

    let job_req: JobRequest = request;
    let resp = client
        .post("jobs")
        .json(&job_req)
        .send()
        .await
        .map_err(|e| CommandError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(CommandError::Network(format!("Server Error: {}", resp.status())));
    }

    let body: JobResponse = resp.json().await.map_err(|e| CommandError::Network(e.to_string()))?;
    Ok(body.job_id)
}

#[tauri::command]
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

    let path = format!("jobs/{}/status", job_id);
    let resp = client.get(&path).send().await.map_err(|e| CommandError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(CommandError::Network(format!("Server Error: {}", resp.status())));
    }

    let status: JobStatus = resp.json().await.map_err(|e| CommandError::Network(e.to_string()))?;
    Ok(JobStatusUpdate {
        active_nodes: status.active_nodes,
        best_score: status.best_score.unwrap_or(0.0),
        best_layout: status.best_layout.unwrap_or_default(),
    })
}

#[tauri::command]
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

        let data_dir = get_data_dir(&app).map_err(|e| CommandError::Internal(e))?;
        
        // Spawn sidecar
        let (mut _rx, child) = app
            .shell()
            .sidecar("keyforge-agent")
            .map_err(|e| CommandError::Internal(format!("Failed to find sidecar: {}", e)))?
            .args(["worker", "--hive", &hive_url, "--data-dir", &data_dir.to_string_lossy()])
            .spawn()
            .map_err(|e| CommandError::Internal(format!("Failed to spawn worker: {}", e)))?;

        *child_guard = Some(child);
        Ok("Worker started".into())
    } else {
        if let Some(child) = child_guard.take() {
            child
                .kill()
                .map_err(|e| CommandError::Internal(format!("Failed to kill worker: {}", e)))?;
            Ok("Worker stopped".into())
        } else {
            Ok("Worker not running".into())
        }
    }
}

// HELPER: Convert ScoringWeights to Rubric
fn weights_to_rubric(w: &keyforge_model::config::ScoringWeights) -> keyforge_model::Rubric {
    keyforge_model::Rubric {
        finger_effort: w.get_finger_penalty_scale(),
        travel_lat: w.weight_lateral_travel,
        travel_vert: w.weight_vertical_travel,
        sfb_base: w.penalty_sfb_base,
        sfb_lateral: w.penalty_sfb_lateral,
        sfb_lateral_weak: w.penalty_sfb_lateral_weak,
        sfb_diagonal: w.penalty_sfb_diagonal,
        sfb_long: w.penalty_sfb_long,
        threshold_sfb_long_row_diff: w.threshold_sfb_long_row_diff,
        penalty_scissor: w.penalty_scissor,
        threshold_scissor_row_diff: w.threshold_scissor_row_diff,
        redirect: w.penalty_redirect,
        roll_bonus: w.bonus_inward_roll, 
        trigram_coverage: w.trigram_coverage,
        trigram_limit: w.loader_trigram_limit,
    }
}

// HELPER: Parse pinned keys string
fn parse_pinned_keys(pinned_str: &str, registry: &keyforge_model::KeycodeRegistry, count: usize) -> Vec<Option<KeyCode>> {
    let mut pinned = vec![None; count];
    // If string has spaces, treat as space-separated tokens
    if pinned_str.contains(' ') {
        for (i, token) in pinned_str.split_whitespace().enumerate() {
            if i >= count { break; }
            if let Some(code) = registry.get_code(token) {
                // If it's KC_NO or TRNS, maybe we treat as None?
                // Assuming tokens present are pinned.
                pinned[i] = Some(code);
            }
        }
    } else {
        // Assume direct mapping char -> key
        for (i, c) in pinned_str.chars().enumerate() {
             if i >= count { break; }
             let s = c.to_string();
             if let Some(code) = registry.get_code(&s) {
                 pinned[i] = Some(code);
             }
        }
    }
    pinned
}

struct TauriProgressCallback {
    window: Window,
    stop_flag: Arc<Mutex<bool>>,
    last_emit: Mutex<std::time::Instant>,
    registry: Arc<keyforge_model::KeycodeRegistry>,
}

impl ProgressCallback for TauriProgressCallback {
    fn on_progress(&self, epoch: usize, score: f32, layout: &[KeyCode], ips: f32) -> bool {
        if let Ok(flag) = self.stop_flag.lock() {
            if *flag { return false; }
        }
        
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
            let _ = self.window.emit("search_update", update);
        }
        true
    }
}

#[tauri::command]
pub async fn cmd_start_search(
    window: Window,
    state: tauri::State<'_, SessionState>,
    search_state: tauri::State<'_, SearchState>,
    request: StartSearchRequest,
) -> Result<String, CommandError> {
    // 1. Get Active Job Context
    let (definition, corpora) = {
        let job_guard = state.active_job.read().await;
        let job = job_guard.as_ref().ok_or_else(|| CommandError::Internal("No active job found".into()))?;
        (job.definition.clone(), job.corpora.clone())
    };

    // 2. Load Assets
    let _kb_asset = state.assets.load_keyboard(&definition.meta.name).await
        .map_err(|e| CommandError::Internal(format!("Failed to load keyboard: {}", e)))?;
        
    let corpus = state.assets.load_corpus(&corpora).await
        .map_err(|e| CommandError::Internal(format!("Failed to load corpus: {}", e)))?;
        
    let keycodes = state.assets.load_keycodes("default").await
        .unwrap_or_else(|_| Arc::new(keyforge_model::KeycodeRegistry::new_with_defaults()));

    // 3. Prepare Engine Request
    let rubric = Arc::new(weights_to_rubric(&request.weights));
    
    // Construct runtime Keyboard from definition
    let runtime_kb = Arc::new(keyforge_model::Keyboard::new(definition.geometry.keys.clone(), definition.geometry.home_row)
        .map_err(|e| CommandError::Internal(e.to_string()))?);
    
    let pinned = parse_pinned_keys(&request.pinned_keys, &keycodes, runtime_kb.count());
    
    let search_config = keyforge_model::SearchConfig::Annealing {
        steps: request.search_params.search_steps,
        start_temp: request.search_params.temp_max,
        end_temp: request.search_params.temp_min,
        seed: request.search_params.seed.unwrap_or_else(|| fastrand::u64(..)),
        patience: request.search_params.search_patience,
        reheats: request.search_params.reheats,
        reheat_factor: request.search_params.reheat_factor,
    };

    // Reset stop flag
    if let Ok(mut flag) = search_state.stop_flag.lock() {
        *flag = false;
    }

    // 4. Spawn Optimization Task
    let stop_flag = search_state.stop_flag.clone();
    let window_handle = window.clone();
    let keycodes_handle = keycodes.clone();

    // Use std::thread::spawn for CPU-bound task to avoid blocking Tokio runtime
    std::thread::spawn(move || {
        let req = EngineRequest {
            keyboard: runtime_kb,
            corpus,
            rubric,
            config: search_config,
            initial_layout: None,
            pinned_keys: pinned,
            cost_overrides: vec![],
        };
        
        let callback = TauriProgressCallback {
            window: window_handle.clone(),
            stop_flag,
            last_emit: Mutex::new(std::time::Instant::now()),
            registry: keycodes_handle,
        };
        
        match optimize_with_callback(&req, callback) {
            Ok(result) => {
                 let _ = window_handle.emit("search_finished", result);
            },
            Err(e) => {
                 let _ = window_handle.emit("search_error", e.to_string());
            }
        }
    });

    Ok("Search started".into())
}

#[tauri::command]
pub fn cmd_stop_search(search_state: tauri::State<'_, SearchState>) {
    if let Ok(mut flag) = search_state.stop_flag.lock() {
        *flag = true;
    }
}