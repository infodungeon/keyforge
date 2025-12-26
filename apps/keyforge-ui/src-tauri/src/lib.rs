use state::{AssetCache, LocalWorkerState, SearchState, SessionState};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::RwLock;

pub mod commands;
pub mod error;
pub mod models;
pub mod state;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            let data_dir = utils::get_data_dir(app.handle())
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;

            let asset_cache = AssetCache::new(data_dir);

            app.manage(SessionState {
                active: Arc::new(RwLock::new(None)),
                assets: Arc::new(asset_cache),
            });

            Ok(())
        })
        .manage(LocalWorkerState {
            child: Arc::new(Mutex::new(None)),
        })
        .manage(SearchState {
            stop_flag: Arc::new(Mutex::new(false)),
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
            commands::system::cmd_check_hive_health
        ])
        .on_window_event(|window, event| {
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
