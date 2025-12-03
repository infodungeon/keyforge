use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;

pub struct LocalWorkerState {
    pub child: Arc<Mutex<Option<CommandChild>>>,
}

pub struct SearchState {
    pub stop_flag: Arc<Mutex<bool>>,
}