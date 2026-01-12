// apps/keyforge-ui/src-tauri/src/lib.rs

//! # KeyForge UI Backend
//!
//! Rust backend for the KeyForge Tauri application. This crate handles 
//! state management, background search workers, and bridges frontend 
//! requests to core KeyForge libraries via Tauri commands.

pub use state::{AssetCache, LocalWorkerState, SearchState, SessionState};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::RwLock;
use keyforge_infra::{initialize_workspace, InitMode};

/// Command handlers for Tauri invoke calls.
pub mod commands;
/// Shared error types for command results.
pub mod error;
/// Data models for frontend communication.
pub mod models;
/// Application state management and synchronization.
pub mod state;
/// Internal utility functions.
pub mod utils;
/// Agent Runner
pub mod runner;

/// The main entry point for the KeyForge UI application.
///
/// This function initializes logging, sets up the Tauri builder, configures
/// plugins, and establishes the global application state including the 
/// asset cache and worker coordination.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let data_dir = utils::get_data_dir(app.handle())
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;

            tracing::info!("Initializing workspace at: {:?}", data_dir);
            
            // Self-Healing Initialization
            if let Err(e) = initialize_workspace(&data_dir, InitMode::Create) {
                tracing::error!("Workspace initialization error: {}", e);
            }

            // Start Embedded Asset Server
            // This allows the frontend (and potentially local agents) to fetch assets via HTTP/3001
            // mirroring the Hive architecture locally.
            let provider = Arc::new(keyforge_infra::FsProvider::new(data_dir.clone()));
            let asset_app = keyforge_assets::create_app(provider);
            
            tauri::async_runtime::spawn(async move {
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3001));
                match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => {
                         tracing::info!("🚀 Embedded Asset Server listening on http://{}", addr);
                         if let Err(e) = axum::serve(listener, asset_app).await {
                             tracing::error!("Asset server error: {}", e);
                         }
                    }
                    Err(e) => {
                         tracing::warn!("Failed to bind asset server to 3001: {}. Is another instance running?", e);
                    }
                }
            });

            let asset_cache = AssetCache::new(data_dir);

            app.manage(SessionState {
                active_job: Arc::new(RwLock::new(None)),
                assets: Arc::new(asset_cache),
                client: Arc::new(RwLock::new(None)),
            });

            Ok(())
        })
        .manage(LocalWorkerState {
            child: Arc::new(Mutex::new(None)),
        })
        .manage(SearchState {
            stop_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Config
            commands::config::cmd_get_default_config,
            commands::config::cmd_get_keycodes,
            commands::config::cmd_get_ui_categories,
            // Library
            commands::library::cmd_list_keyboards,
            commands::library::cmd_list_keymap_extras,
            commands::library::cmd_get_loaded_layouts,
            commands::library::cmd_get_all_layouts_scoped,
            commands::library::cmd_get_keyboard_geometry,
            commands::library::cmd_save_user_layout,
            commands::library::cmd_delete_user_layout,
            commands::library::cmd_submit_user_layout,
            commands::library::cmd_parse_kle,
            commands::library::cmd_export_to_kle,
            commands::library::cmd_save_keyboard,
            commands::library::cmd_export_firmware,
            commands::library::cmd_safe_write_file,
            // Analysis
            commands::analysis::cmd_list_corpora,
            commands::analysis::cmd_get_corpus_stats,
            commands::analysis::cmd_list_cost_matrices,
            commands::analysis::cmd_load_dataset,
            commands::analysis::cmd_validate_layout,
            commands::analysis::cmd_get_layout_stats,
            commands::analysis::cmd_get_smart_swaps,
            // Search
            commands::search::cmd_dispatch_job,
            commands::search::cmd_poll_hive_status,
            commands::search::cmd_toggle_local_worker,
            commands::search::cmd_start_search,
            commands::search::cmd_stop_search,
            // Sync
            commands::sync::cmd_sync_data,
            commands::sync::cmd_bootstrap_assets,
            // Arena
            commands::arena::cmd_get_typing_words,
            commands::arena::cmd_save_biometrics,
            commands::arena::cmd_load_user_stats,
            commands::arena::cmd_generate_personal_profile,
            commands::arena::cmd_reset_user_stats,
            commands::arena::cmd_get_corpus_bigrams,
            // System
            commands::system::cmd_get_system_health,
            commands::system::cmd_check_hive_health,
            commands::system::cmd_set_hive_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
