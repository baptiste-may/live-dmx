//! Physical USB DMX interfaces: serial port discovery, Enttec framing and sending.
//!
//! Also exposes the Tauri commands the frontend uses to list interfaces. See
//! `docs/contributing/dmx-output.md` for the details.

/// Serial port scanning and the fictional test interfaces.
pub mod discovery;
/// Serial port sender using Enttec DMX USB Pro framing.
pub mod sender;
/// Data model of a physical interface surfaced to the frontend.
pub mod types;

pub use discovery::*;
pub use sender::*;
pub use types::*;

/// Tauri command: lists the physical DMX interfaces currently detected.
///
/// # Errors
///
/// Returns `Err` with the message when the serial ports cannot be enumerated.
#[tauri::command]
pub fn get_output_interfaces() -> Result<Vec<PhysicalInterface>, String> {
    discover_physical_interfaces()
}

/// Tauri command: rescans the serial ports and lists the detected interfaces.
///
/// Currently identical to [`get_output_interfaces`]; no cache is involved, the
/// separate name lets the frontend express an explicit refresh.
///
/// # Errors
///
/// Returns `Err` with the message when the serial ports cannot be enumerated.
#[tauri::command]
pub fn scan_output_interfaces() -> Result<Vec<PhysicalInterface>, String> {
    discover_physical_interfaces()
}
