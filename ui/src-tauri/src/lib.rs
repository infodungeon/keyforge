use keyforge_core::api::KeyForgeState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

// 1. Register Modules
pub mod commands;
pub mod models;
pub mod state;
pub mod utils;

// 2. Import State Structs
use state::{LocalWorkerState, SearchState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        // --- State Management ---
        .manage(KeyForgeState::default())
        .manage(LocalWorkerState {
            child: Arc::new(Mutex::new(None)),
        })
        .manage(SearchState {
            stop_flag: Arc::new(Mutex::new(false)),
        })
        // --- Plugins ---
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // --- Command Registration ---
        .invoke_handler(tauri::generate_handler![
            // Config
            commands::config::cmd_get_default_config,
            commands::config::cmd_get_keycodes,
            commands::config::cmd_get_ui_categories,
            // Library (Layouts & Geometry)
            commands::library::cmd_list_keyboards,
            commands::library::cmd_get_loaded_layouts,
            commands::library::cmd_get_all_layouts_scoped,
            commands::library::cmd_get_keyboard_geometry,
            commands::library::cmd_save_user_layout,
            commands::library::cmd_delete_user_layout,
            commands::library::cmd_submit_user_layout,
            commands::library::cmd_parse_kle,
            commands::library::cmd_save_keyboard,
            // Analysis (Scoring & Datasets)
            commands::analysis::cmd_list_corpora,
            commands::analysis::cmd_import_corpus,
            commands::analysis::cmd_load_dataset,
            commands::analysis::cmd_validate_layout,
            // Search / Optimize (Hive & Local)
            commands::search::cmd_dispatch_job,
            commands::search::cmd_poll_hive_status,
            commands::search::cmd_toggle_local_worker,
            commands::search::cmd_start_search,
            commands::search::cmd_stop_search,
            // Sync
            commands::sync::cmd_sync_data,
            // Arena (Typing Test)
            commands::arena::cmd_get_typing_words,
            commands::arena::cmd_save_biometrics
        ])
        .on_window_event(|window, event| {
            // Cleanup child processes on exit
            if let tauri::WindowEvent::Destroyed = event {
                let maybe_child = {
                    let state = window.state::<LocalWorkerState>();
                    let mut guard = state.child.lock().unwrap();
                    guard.take()
                };
                if let Some(child) = maybe_child {
                    let _ = child.kill();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
