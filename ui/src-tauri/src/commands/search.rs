// ===== keyforge/ui/src-tauri/src/commands/search.rs =====
use crate::models::{
    JobStatusUpdate, PopulationResponse, RegisterJobRequest, RegisterJobResponse,
    StartSearchRequest,
};
use crate::state::{LocalWorkerState, SearchState};
use crate::utils::TauriBridge;
use keyforge_core::api::KeyForgeState;
use keyforge_core::config::Config;
use keyforge_core::optimizer::{OptimizationOptions, Optimizer};
use reqwest::Client;
use std::sync::Arc;
use tauri::{AppHandle, Window};
use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub async fn cmd_dispatch_job(
    state: tauri::State<'_, KeyForgeState>,
    hive_url: String,
    request: RegisterJobRequest,
) -> Result<String, String> {
    let client = Client::new();
    {
        // Acquire read lock on sessions to ensure we have a valid session
        let sessions = state.sessions.read().map_err(|e| e.to_string())?;
        if sessions.get("primary").is_none() {
            return Err("No local geometry loaded to validate against".into());
        }
    }

    let res = client
        .post(format!("{}/jobs", hive_url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network Error: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Hive rejected job: {}", res.status()));
    }

    let body: RegisterJobResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(body.job_id)
}

#[tauri::command]
pub async fn cmd_poll_hive_status(
    hive_url: String,
    job_id: String,
) -> Result<JobStatusUpdate, String> {
    let client = Client::new();
    let url = format!("{}/jobs/{}/population", hive_url, job_id);
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err("Job not found or Hive error".into());
    }

    let pop: PopulationResponse = res.json().await.map_err(|e| e.to_string())?;

    if let Some(best) = pop.layouts.first() {
        Ok(JobStatusUpdate {
            active_nodes: 0, // In a real implementation, we'd poll a different endpoint for active node count
            best_score: 0.0, // Score is usually embedded in the layout list or separate metadata, simplified here
            best_layout: best.clone(),
        })
    } else {
        Err("No results yet".into())
    }
}

#[tauri::command]
pub async fn cmd_toggle_local_worker(
    app: AppHandle,
    state: tauri::State<'_, LocalWorkerState>,
    enabled: bool,
    hive_url: String,
) -> Result<String, String> {
    let mut child_guard = state.child.lock().unwrap();

    if let Some(child) = child_guard.take() {
        tracing::info!("🛑 Stopping local worker...");
        let _ = child.kill();
    }

    if enabled {
        tracing::info!("🟢 Spawning local worker (Background Mode)...");
        let command = app
            .shell()
            .sidecar("keyforge-node")
            .map_err(|e| e.to_string())?
            .args(["work", "--hive", &hive_url, "--background"]);

        let (mut rx, child) = command.spawn().map_err(|e| e.to_string())?;

        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
                    tracing::debug!("[Worker]: {}", String::from_utf8_lossy(&line));
                }
            }
        });

        *child_guard = Some(child);
        return Ok("Worker Started".to_string());
    }
    Ok("Worker Stopped".to_string())
}

#[tauri::command]
pub async fn cmd_start_search(
    window: Window,
    state: tauri::State<'_, KeyForgeState>,
    search_state: tauri::State<'_, SearchState>,
    request: StartSearchRequest,
) -> Result<String, String> {
    // 1. Prepare Environment (Scorer & Registry)
    let (scorer_arc, registry_arc) = {
        let sessions = state.sessions.read().map_err(|e| e.to_string())?;

        let session = sessions.get("primary").ok_or("Session not loaded")?;

        // Clone scorer to apply new weights for this run
        let mut scorer = session.scorer.clone();
        scorer.weights = request.weights;

        (Arc::new(scorer), Arc::new(session.registry.clone()))
    };

    // 2. Configure Optimizer
    let search_params = request.search_params;

    // Create a temporary config to utilize From implementation
    let config = Config {
        search: search_params,
        ..Default::default()
    };

    let mut options = OptimizationOptions::from(&config);

    // FIX: Set pinned keys explicitly on options, not via SearchParams
    options.pinned_keys = request.pinned_keys;

    let optimizer = Optimizer::new(scorer_arc, options);

    // 3. Reset Stop Flag
    *search_state.stop_flag.lock().unwrap() = false;

    let bridge = TauriBridge {
        window,
        stop_signal: search_state.stop_flag.clone(),
    };

    tracing::info!("Starting Deep Optimization...");

    // 4. Run Optimization (Blocking inside async runtime)
    let result = tauri::async_runtime::spawn_blocking(move || optimizer.run(None, bridge))
        .await
        .map_err(|e| e.to_string())?;

    // 5. Format Result
    let layout_str = result
        .layout
        .iter()
        .map(|&code| registry_arc.get_label(code))
        .collect::<Vec<String>>()
        .join(" ");

    Ok(layout_str)
}

#[tauri::command]
pub fn cmd_stop_search(search_state: tauri::State<SearchState>) {
    *search_state.stop_flag.lock().unwrap() = true;
}
