//! Desktop binary of live-dmx: a thin wrapper that delegates to `live_dmx_lib::run`.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Desktop entry point; all application logic lives in the `live_dmx_lib` crate.
fn main() {
    live_dmx_lib::run()
}
