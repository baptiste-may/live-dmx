//! Build script: runs the Tauri build step (capabilities, config and resource embedding).

/// Invokes `tauri_build::build` so Tauri can generate its build-time context.
fn main() {
    tauri_build::build()
}
