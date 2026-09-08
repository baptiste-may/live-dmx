use tauri::{AppHandle, State};

use crate::show::state::AppState;
use crate::show::types::UniverseOutputConfig;

#[tauri::command]
pub fn show_output_get_universes(
    state: State<'_, AppState>,
) -> Result<Vec<UniverseOutputConfig>, String> {
    let show = state.show.lock().map_err(|e| e.to_string())?;
    Ok(show.output.universes.clone())
}

#[tauri::command]
pub fn show_output_set_target(
    state: State<'_, AppState>,
    app: AppHandle,
    universe: u32,
    output: String,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        if let Some(target) = show
            .output
            .universes
            .iter_mut()
            .find(|u| u.universe == universe)
        {
            target.output = output;
            Ok(())
        } else {
            Err(format!("Universe {universe} not found"))
        }
    })
}

#[tauri::command]
pub fn show_output_set_enabled(
    state: State<'_, AppState>,
    app: AppHandle,
    universe: u32,
    enabled: bool,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        if let Some(target) = show
            .output
            .universes
            .iter_mut()
            .find(|u| u.universe == universe)
        {
            target.enabled = enabled;
            Ok(())
        } else {
            Err(format!("Universe {universe} not found"))
        }
    })
}

#[tauri::command]
pub fn show_output_add_universe(state: State<'_, AppState>, app: AppHandle) -> Result<u32, String> {
    state.apply_mutation(&app, |show| {
        let max_universe = show
            .output
            .universes
            .iter()
            .map(|u| u.universe)
            .max()
            .unwrap_or(0);
        let new_universe = max_universe + 1;
        show.output.universes.push(UniverseOutputConfig {
            universe: new_universe,
            output: "none".to_string(),
            enabled: true,
        });
        Ok(new_universe)
    })
}

#[tauri::command]
pub fn show_output_remove_universe(
    state: State<'_, AppState>,
    app: AppHandle,
    universe: u32,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        if show.output.universes.len() <= 1 {
            return Err("Cannot remove the last universe".to_string());
        }
        if let Some(pos) = show
            .output
            .universes
            .iter()
            .position(|u| u.universe == universe)
        {
            show.output.universes.remove(pos);
            Ok(())
        } else {
            Err(format!("Universe {universe} not found"))
        }
    })
}
