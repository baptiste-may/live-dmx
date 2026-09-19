use std::io::Write;

use tauri::{AppHandle, State};

use crate::show::state::{emit_progress, emit_show_changed, AppState};
use crate::show::types::{
    clean_patched_fixture, EmbeddedGdtfDefinition, PatchGroupsState, PatchedFixture,
    ShowChangesState, ShowData, ShowFileContent, ShowIdentityState, ShowOutputState,
    ShowPatchState, ShowStatus,
};

pub fn enrich_fixture_modes_bounds(
    fixtures: &mut [PatchedFixture],
    fixture_definitions: &std::collections::HashMap<String, EmbeddedGdtfDefinition>,
) {
    let generic_fixtures = crate::fixtures::generic::get_generic_fixtures();
    for fix in fixtures {
        if fix.mode.get("panBounds").is_some() && fix.mode.get("tiltBounds").is_some() {
            continue;
        }
        let fix_name = fix
            .fixture_type
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mode_name = fix.mode.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let mut found_pan = None;
        let mut found_tilt = None;

        // 1. Check embedded fixture definitions
        if let Some(def) = fixture_definitions.values().find(|d| {
            d.name.eq_ignore_ascii_case(fix_name)
                || d.short_name
                    .as_deref()
                    .unwrap_or("")
                    .eq_ignore_ascii_case(fix_name)
        }) {
            if let Some(m) = def.modes.iter().find(|m| {
                m.name.eq_ignore_ascii_case(mode_name)
                    || m.short_name
                        .as_deref()
                        .unwrap_or("")
                        .eq_ignore_ascii_case(mode_name)
            }) {
                found_pan = m.pan_bounds;
                found_tilt = m.tilt_bounds;
            }
        }

        // 2. Check generic fixtures
        if found_pan.is_none() || found_tilt.is_none() {
            if let Some(g) = generic_fixtures.iter().find(|g| {
                g.name.eq_ignore_ascii_case(fix_name)
                    || g.short_name
                        .as_deref()
                        .unwrap_or("")
                        .eq_ignore_ascii_case(fix_name)
            }) {
                if let Some(m) = g.modes.iter().find(|m| {
                    m.name.eq_ignore_ascii_case(mode_name)
                        || m.short_name
                            .as_deref()
                            .unwrap_or("")
                            .eq_ignore_ascii_case(mode_name)
                }) {
                    if found_pan.is_none() {
                        found_pan = m.pan_bounds;
                    }
                    if found_tilt.is_none() {
                        found_tilt = m.tilt_bounds;
                    }
                }
            }
        }

        if let Some(obj) = fix.mode.as_object_mut() {
            if obj.get("panBounds").is_none() {
                if let Some(pb) = found_pan {
                    obj.insert("panBounds".to_string(), serde_json::json!(pb));
                }
            }
            if obj.get("tiltBounds").is_none() {
                if let Some(tb) = found_tilt {
                    obj.insert("tiltBounds".to_string(), serde_json::json!(tb));
                }
            }
        }
    }
}

#[tauri::command]
pub fn show_get_data(state: State<'_, AppState>) -> Result<ShowData, String> {
    let show = state.show.lock().map_err(|e| e.to_string())?;
    let mut response = show.clone();
    enrich_fixture_modes_bounds(&mut response.patch.fixtures, &response.fixture_definitions);
    Ok(response)
}

#[tauri::command]
pub fn show_create(state: State<'_, AppState>, app: AppHandle) -> Result<ShowData, String> {
    log::info!(target: "live-dmx::show", "creating new show");
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    *show = ShowData {
        status: ShowStatus::Loaded,
        identity: ShowIdentityState::default(),
        file: ShowChangesState::default(),
        patch: ShowPatchState::default(),
        groups: PatchGroupsState::default(),
        output: ShowOutputState::default(),
        fixture_definitions: std::collections::HashMap::new(),
    };
    history.reset_with_saved(show.snapshot());
    show.file.is_dirty = false;
    show.file.can_undo = false;
    show.file.can_redo = false;
    state.reset_live_engine()?;
    emit_show_changed(&app, &show);
    Ok(show.clone())
}

fn write_show_file(show: &ShowData, path: &str) -> Result<(), String> {
    let clean_fixtures: Vec<_> = show
        .patch
        .fixtures
        .iter()
        .map(clean_patched_fixture)
        .collect();

    let content = ShowFileContent {
        identity: show.identity.clone(),
        patch: ShowPatchState {
            fixtures: clean_fixtures,
        },
        groups: show.groups.clone(),
        output: show.output.clone(),
        fixture_definitions: show.fixture_definitions.clone(),
    };

    let json_str = serde_json::to_string_pretty(&content).map_err(|e| e.to_string())?;

    let mut suffix = [0u8; 8];
    getrandom::fill(&mut suffix).map_err(|e| e.to_string())?;
    let suffix: String = suffix.iter().map(|b| format!("{b:02x}")).collect();
    let tmp_path = format!("{path}.tmp.{suffix}");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .map_err(|e| e.to_string())?;
    let write_result = file.write_all(json_str.as_bytes());
    drop(file);
    if let Err(err) = write_result {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(err.to_string());
    }

    std::fs::rename(&tmp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_path);
        e.to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub fn show_save(
    state: State<'_, AppState>,
    app: AppHandle,
    path: String,
) -> Result<ShowData, String> {
    log::info!(target: "live-dmx::show", "saving show to {path}");
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;

    show.status = ShowStatus::Saving;
    emit_show_changed(&app, &show);

    if let Err(err) = write_show_file(&show, &path) {
        show.status = ShowStatus::Loaded;
        emit_show_changed(&app, &show);
        return Err(err);
    }

    show.file.path = Some(path);
    history.mark_saved(show.snapshot());
    show.file.is_dirty = false;
    show.file.can_undo = history.can_undo();
    show.file.can_redo = history.can_redo();
    show.status = ShowStatus::Loaded;

    emit_show_changed(&app, &show);
    Ok(show.clone())
}

#[tauri::command]
pub fn show_load(
    state: State<'_, AppState>,
    app: AppHandle,
    path: String,
) -> Result<ShowData, String> {
    log::info!(target: "live-dmx::show", "loading show from {path}");
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;

    emit_progress(&app, ShowStatus::Loading, 5, "Reading show file...");
    let json_str = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    emit_progress(&app, ShowStatus::Loading, 25, "Parsing show data...");
    let content: ShowFileContent = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;

    emit_progress(&app, ShowStatus::Loading, 40, "Cleaning fixtures...");
    let fixtures = content.patch.fixtures;
    let total = fixtures.len().max(1);
    let mut clean_fixtures = Vec::with_capacity(fixtures.len());
    for (i, fixture) in fixtures.into_iter().enumerate() {
        clean_fixtures.push(clean_patched_fixture(&fixture));
        if i % 20 == 0 || i + 1 == total {
            let pct = 40 + ((i + 1) * 30 / total) as u8;
            emit_progress(&app, ShowStatus::Loading, pct, "Cleaning fixtures...");
        }
    }

    emit_progress(&app, ShowStatus::Loading, 80, "Enriching fixture data...");
    enrich_fixture_modes_bounds(&mut clean_fixtures, &content.fixture_definitions);

    emit_progress(&app, ShowStatus::Loading, 92, "Applying show data...");
    *show = ShowData {
        status: ShowStatus::Loaded,
        identity: content.identity,
        file: ShowChangesState {
            path: Some(path),
            is_dirty: false,
            is_unsaved_changes_modal_open: false,
            can_undo: false,
            can_redo: false,
        },
        patch: ShowPatchState {
            fixtures: clean_fixtures,
        },
        groups: content.groups,
        output: content.output,
        fixture_definitions: content.fixture_definitions,
    };
    history.reset_with_saved(show.snapshot());
    state.reset_live_engine()?;

    emit_progress(&app, ShowStatus::Loaded, 100, "");
    emit_show_changed(&app, &show);
    Ok(show.clone())
}

#[tauri::command]
pub fn show_import_mvr(
    state: State<'_, AppState>,
    app: AppHandle,
    path: String,
) -> Result<ShowData, String> {
    let imported = crate::show::mvr::import_mvr_file(&path, Some(&app))?;
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;

    *show = imported;
    history.reset_empty();
    state.reset_live_engine()?;

    emit_show_changed(&app, &show);
    Ok(show.clone())
}

#[tauri::command]
pub fn show_unload(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    *show = ShowData::default();
    history.reset_empty();
    state.reset_live_engine()?;
    emit_show_changed(&app, &show);
    Ok(())
}

#[tauri::command]
pub fn show_undo(state: State<'_, AppState>, app: AppHandle) -> Result<ShowData, String> {
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;

    if let Some(prev_snapshot) = history.undo_stack.pop() {
        let current_snapshot = show.snapshot();
        let patch_or_groups_changed = current_snapshot.patch != prev_snapshot.patch
            || current_snapshot.groups.groups != prev_snapshot.groups.groups;

        history.redo_stack.push(current_snapshot);

        show.restore_snapshot(prev_snapshot);

        let current = show.snapshot();
        show.file.is_dirty = history.is_dirty(&current);
        show.file.can_undo = history.can_undo();
        show.file.can_redo = history.can_redo();

        if patch_or_groups_changed {
            state.reset_live_engine()?;
        }

        emit_show_changed(&app, &show);
    }
    Ok(show.clone())
}

#[tauri::command]
pub fn show_redo(state: State<'_, AppState>, app: AppHandle) -> Result<ShowData, String> {
    let mut show = state.show.lock().map_err(|e| e.to_string())?;
    let mut history = state.history.lock().map_err(|e| e.to_string())?;

    if let Some(next_snapshot) = history.redo_stack.pop() {
        let current_snapshot = show.snapshot();
        let patch_or_groups_changed = current_snapshot.patch != next_snapshot.patch
            || current_snapshot.groups.groups != next_snapshot.groups.groups;

        history.undo_stack.push(current_snapshot);

        show.restore_snapshot(next_snapshot);

        let current = show.snapshot();
        show.file.is_dirty = history.is_dirty(&current);
        show.file.can_undo = history.can_undo();
        show.file.can_redo = history.can_redo();

        if patch_or_groups_changed {
            state.reset_live_engine()?;
        }

        emit_show_changed(&app, &show);
    }
    Ok(show.clone())
}

#[tauri::command]
pub fn show_register_fixture_definition(
    state: State<'_, AppState>,
    app: AppHandle,
    definition: crate::show::types::EmbeddedGdtfDefinition,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        let key = format!(
            "{}@{}",
            definition.manufacturer.to_lowercase(),
            definition.name.to_lowercase()
        );
        show.fixture_definitions.insert(key, definition);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_show_file_round_trips_and_leaves_no_temp_file() {
        let show = ShowData::default();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("My Show.ldmx");

        write_show_file(&show, path.to_str().unwrap()).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        let content: ShowFileContent = serde_json::from_str(&raw).unwrap();
        assert_eq!(content.identity, show.identity);

        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "unexpected temp files: {leftovers:?}");
    }
}
