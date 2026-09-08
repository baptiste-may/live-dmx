pub mod discovery;
pub mod sender;
pub mod types;

pub use discovery::*;
pub use sender::*;
pub use types::*;

#[tauri::command]
pub fn get_output_interfaces() -> Result<Vec<PhysicalInterface>, String> {
    discover_physical_interfaces()
}

#[tauri::command]
pub fn scan_output_interfaces() -> Result<Vec<PhysicalInterface>, String> {
    discover_physical_interfaces()
}
