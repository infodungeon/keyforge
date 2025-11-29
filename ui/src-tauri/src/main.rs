#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // This calls the run() function in lib.rs
    ui_lib::run();
}
