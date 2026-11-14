//! Show identity command (the show display name).

use tauri::{AppHandle, State};

use super::state::AppState;

/// Tauri command: renames the show.
///
/// Applied through `AppState::apply_mutation`, so it is undoable, marks the
/// show dirty and emits `show-state-changed`.
///
/// # Errors
///
/// Returns an error if a state mutex is poisoned.
#[tauri::command]
pub fn show_identity_update(
    state: State<'_, AppState>,
    app: AppHandle,
    display_name: String,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        show.identity.display_name = display_name;
        Ok(())
    })
}
