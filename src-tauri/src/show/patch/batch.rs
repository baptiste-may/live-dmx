use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::show::{AppState, FixtureGroup, PatchRequest, PatchedFixture};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchPatchRequest {
    pub name: String,
    pub fixture_type: serde_json::Value,
    pub mode: serde_json::Value,
    pub universe: u32,
    pub start_address: u32,
    pub count: u32,
    pub channel_count: u32,
}

pub fn extract_standard_name(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() > 1 && parts.last().and_then(|p| p.parse::<u32>().ok()).is_some() {
        parts[..parts.len() - 1].join(" ")
    } else {
        name.to_string()
    }
}

#[tauri::command]
pub fn show_patch_fixture(
    state: State<'_, AppState>,
    app: AppHandle,
    request: PatchRequest,
    group_id: u32,
) -> Result<PatchedFixture, String> {
    state.apply_mutation(&app, |show| {
        let channel_count = request
            .mode
            .get("channels")
            .and_then(|c| c.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(1)
            .max(1);

        let end_address = request.address + channel_count - 1;
        if request.universe < 1 || request.universe > 512 || request.address < 1 || end_address > 512 {
            return Err(format!(
                "Address out of bounds: Universe {}, Address {}..{}",
                request.universe, request.address, end_address
            ));
        }

        for existing in &show.patch.fixtures {
            if existing.universe == request.universe {
                let existing_channels = get_fixture_channel_count(existing) as u32;
                let existing_start = existing.address;
                let existing_end = existing_start + existing_channels - 1;

                if existing_start <= end_address && request.address <= existing_end {
                    return Err(format!(
                        "Address collision on universe {}, address {}: overlaps with fixture '{}' ({}-{})",
                        request.universe, request.address, existing.name, existing_start, existing_end
                    ));
                }
            }
        }

        let next_id = if show.patch.fixtures.is_empty() {
            1
        } else {
            show.patch.fixtures.iter().map(|f| f.id).max().unwrap_or(0) + 1
        };

        let patched = PatchedFixture {
            id: next_id,
            groups: vec![group_id],
            name: request.name,
            fixture_type: request.fixture_type,
            mode: request.mode,
            universe: request.universe,
            address: request.address,
        };

        show.patch.fixtures.push(patched.clone());
        Ok(patched)
    })
}

pub fn check_batch_patch_collision(
    fixtures: &[PatchedFixture],
    request: &BatchPatchRequest,
) -> Result<Vec<(u32, u32)>, String> {
    let channel_count = request.channel_count.max(1) as u64;
    let mut current_abs_address = u64::from(request.universe.saturating_sub(1)) * 512
        + u64::from(request.start_address.saturating_sub(1));

    let mut planned_fixtures = Vec::new();

    for _ in 0..request.count {
        let offset_in_universe = current_abs_address % 512;
        if offset_in_universe + channel_count > 512 {
            current_abs_address += 512 - offset_in_universe;
        }

        let universe = ((current_abs_address / 512) + 1) as u32;
        let address = ((current_abs_address % 512) + 1) as u32;
        let end_address = address + channel_count as u32 - 1;

        if universe > 512 || end_address > 512 {
            return Err(format!(
                "Patch exceeds universe bounds (universe {}, address {})",
                universe, address
            ));
        }

        for existing in fixtures {
            if existing.universe == universe {
                let existing_channels = get_fixture_channel_count(existing) as u32;
                let existing_start = existing.address;
                let existing_end = existing_start + existing_channels - 1;

                if existing_start <= end_address && address <= existing_end {
                    return Err(format!(
                        "Address collision on universe {}, address {}: overlaps with fixture '{}' ({}-{})",
                        universe, address, existing.name, existing_start, existing_end
                    ));
                }
            }
        }

        planned_fixtures.push((universe, address));
        current_abs_address += channel_count;
    }

    Ok(planned_fixtures)
}

#[tauri::command]
pub fn show_patch_batch(
    state: State<'_, AppState>,
    app: AppHandle,
    request: BatchPatchRequest,
) -> Result<Vec<PatchedFixture>, String> {
    state.apply_mutation(&app, |show| {
        let planned_fixtures = check_batch_patch_collision(&show.patch.fixtures, &request)?;

        let base_name = extract_standard_name(&request.name);

        let max_group_id = show.groups.groups.iter().map(|g| g.id).max().unwrap_or(0);
        let group_id = max_group_id + 1;
        show.groups.groups.push(FixtureGroup {
            id: group_id,
            name: base_name.clone(),
        });

        let start_id = if show.patch.fixtures.is_empty() {
            1
        } else {
            show.patch.fixtures.iter().map(|f| f.id).max().unwrap_or(0) + 1
        };

        let mut created_fixtures = Vec::new();
        for (i, (universe, address)) in planned_fixtures.into_iter().enumerate() {
            let fixture_name = if request.count == 1 {
                request.name.clone()
            } else {
                format!("{} {}", base_name, i + 1)
            };

            let patched = PatchedFixture {
                id: start_id + i as u32,
                groups: vec![group_id],
                name: fixture_name,
                fixture_type: request.fixture_type.clone(),
                mode: request.mode.clone(),
                universe,
                address,
            };

            show.patch.fixtures.push(patched.clone());
            created_fixtures.push(patched);
        }

        Ok(created_fixtures)
    })
}

#[tauri::command]
pub fn show_patch_remove_fixtures(
    state: State<'_, AppState>,
    app: AppHandle,
    ids: Vec<u32>,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        show.patch.fixtures.retain(|f| !ids.contains(&f.id));
        Ok(())
    })
}

#[tauri::command]
pub fn show_patch_update_name(
    state: State<'_, AppState>,
    app: AppHandle,
    ids: Vec<u32>,
    name: String,
) -> Result<(), String> {
    state.apply_mutation(&app, |show| {
        let base_name = extract_standard_name(&name);
        let count = ids.len();
        let mut updated_count = 0;

        for id in &ids {
            if let Some(fixture) = show.patch.fixtures.iter_mut().find(|f| f.id == *id) {
                if count == 1 {
                    fixture.name = name.clone();
                } else {
                    fixture.name = format!("{} {}", base_name, updated_count + 1);
                }
                updated_count += 1;
            }
        }
        Ok(())
    })
}

pub fn get_fixture_channel_count(fixture: &PatchedFixture) -> u64 {
    fixture
        .mode
        .get("channels")
        .and_then(|c| c.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(1)
        .max(1)
}

#[tauri::command]
pub fn show_patch_update_address(
    state: State<'_, AppState>,
    app: AppHandle,
    ids: Vec<u32>,
    address: u32,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }

    state.apply_mutation(&app, |show| {
        let start_universe = ids
            .first()
            .and_then(|id| show.patch.fixtures.iter().find(|f| f.id == *id))
            .map(|f| f.universe)
            .unwrap_or(1);

        let mut current_abs_address = u64::from(start_universe.saturating_sub(1)) * 512
            + u64::from(address.saturating_sub(1));

        for id in &ids {
            if let Some(fixture) = show.patch.fixtures.iter_mut().find(|f| f.id == *id) {
                let channel_count = get_fixture_channel_count(fixture);
                let offset_in_universe = current_abs_address % 512;
                if offset_in_universe + channel_count > 512 {
                    current_abs_address += 512 - offset_in_universe;
                }

                fixture.universe = ((current_abs_address / 512) + 1) as u32;
                fixture.address = ((current_abs_address % 512) + 1) as u32;
                current_abs_address += channel_count;
            }
        }
        Ok(())
    })
}

#[tauri::command]
pub fn show_patch_update_universe(
    state: State<'_, AppState>,
    app: AppHandle,
    ids: Vec<u32>,
    universe: u32,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }

    state.apply_mutation(&app, |show| {
        let start_address = ids
            .first()
            .and_then(|id| show.patch.fixtures.iter().find(|f| f.id == *id))
            .map(|f| f.address)
            .unwrap_or(1);

        let mut current_abs_address = u64::from(universe.saturating_sub(1)) * 512
            + u64::from(start_address.saturating_sub(1));

        for id in &ids {
            if let Some(fixture) = show.patch.fixtures.iter_mut().find(|f| f.id == *id) {
                let channel_count = get_fixture_channel_count(fixture);
                let offset_in_universe = current_abs_address % 512;
                if offset_in_universe + channel_count > 512 {
                    current_abs_address += 512 - offset_in_universe;
                }

                fixture.universe = ((current_abs_address / 512) + 1) as u32;
                fixture.address = ((current_abs_address % 512) + 1) as u32;
                current_abs_address += channel_count;
            }
        }
        Ok(())
    })
}

#[tauri::command]
pub fn show_patch_toggle_group(
    state: State<'_, AppState>,
    app: AppHandle,
    ids: Vec<u32>,
    group_id: u32,
) -> Result<(), String> {
    if group_id == 0 || ids.is_empty() {
        return Ok(());
    }

    state.apply_mutation(&app, |show| {
        let all_have_group = ids.iter().all(|id| {
            show.patch
                .fixtures
                .iter()
                .find(|f| f.id == *id)
                .is_some_and(|f| f.groups.contains(&group_id))
        });

        for id in &ids {
            if let Some(fixture) = show.patch.fixtures.iter_mut().find(|f| f.id == *id) {
                if all_have_group {
                    fixture.groups.retain(|&g| g != group_id);
                } else if !fixture.groups.contains(&group_id) {
                    fixture.groups.push(group_id);
                }
            }
        }
        Ok(())
    })
}
