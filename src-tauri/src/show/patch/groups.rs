use tauri::{AppHandle, State};

use crate::show::{AppState, FixtureGroup};

#[tauri::command]
pub fn show_group_create(
    state: State<'_, AppState>,
    app: AppHandle,
    name: String,
) -> Result<u32, String> {
    state.apply_mutation(&app, |show| {
        let max_id = show.groups.groups.iter().map(|g| g.id).max().unwrap_or(0);
        let new_id = max_id + 1;
        show.groups.groups.push(FixtureGroup { id: new_id, name });
        Ok(new_id)
    })
}

#[tauri::command]
pub fn show_group_remove(
    state: State<'_, AppState>,
    app: AppHandle,
    id: u32,
) -> Result<(), String> {
    if id == 0 {
        return Ok(());
    }
    state.apply_mutation(&app, |show| {
        show.groups.groups.retain(|g| g.id != id);
        for f in &mut show.patch.fixtures {
            f.groups.retain(|&gid| gid != id);
        }
        Ok(())
    })
}

#[tauri::command]
pub fn show_group_rename(
    state: State<'_, AppState>,
    app: AppHandle,
    id: u32,
    new_name: String,
) -> Result<(), String> {
    if id == 0 {
        return Ok(());
    }
    state.apply_mutation(&app, |show| {
        if let Some(group) = show.groups.groups.iter_mut().find(|g| g.id == id) {
            group.name = new_name;
        }
        Ok(())
    })
}
