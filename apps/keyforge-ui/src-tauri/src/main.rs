//! `KeyForge` UI Application Entry Point
//!
//! This crate provides the desktop user interface for `KeyForge`, built with Tauri and React.
//! It coordinates between the local search agent, the remote Hive server, and the
//! hardware layout editor.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Main executable entry.
fn main() {
    // This calls the run() function in lib.rs
    ui_lib::run();
}
