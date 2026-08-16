use tauri::{AppHandle, State};

use super::state::AppState;

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
