use tauri::{AppHandle, State};

use crate::show::configure::{
    clamp_group_settings, get_group_color_options, get_group_pan_bounds, get_group_pixel_options,
    get_group_tilt_bounds,
};
use crate::show::{AppState, FixtureGroup, GroupSettings, PatchedFixture};

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
pub fn show_group_update_settings(
    state: State<'_, AppState>,
    app: AppHandle,
    group_id: u32,
    mut settings: GroupSettings,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        if group_id != 0 && !show.groups.groups.iter().any(|g| g.id == group_id) {
            return Err("Group not found".to_string());
        }
        let relevant_fixtures: Vec<&PatchedFixture> = if group_id == 0 {
            show.patch.fixtures.iter().collect()
        } else {
            show.patch
                .fixtures
                .iter()
                .filter(|f| f.groups.contains(&group_id))
                .collect()
        };
        let pan_bounds = get_group_pan_bounds(&relevant_fixtures, &show.fixture_definitions);
        let tilt_bounds = get_group_tilt_bounds(&relevant_fixtures, &show.fixture_definitions);
        let color_options = get_group_color_options(&relevant_fixtures);
        let (_has_pixels, pixel_options, max_pixels) = get_group_pixel_options(&relevant_fixtures);

        clamp_group_settings(
            &mut settings,
            pan_bounds,
            tilt_bounds,
            &color_options,
            &pixel_options,
            max_pixels,
        );

        show.groups.settings.insert(group_id, settings);
        Ok(())
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
        show.groups.settings.remove(&id);
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
