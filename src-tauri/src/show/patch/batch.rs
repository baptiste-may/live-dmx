//! Patch mutations: single and batch fixture patching, renaming, re-addressing and group toggling.
//!
//! Every Tauri command here goes through `AppState::apply_mutation`: the closure edits a
//! `ShowData` under lock, an `Err` rolls the show back untouched, and a real change is pushed on
//! the undo stack, resets the live engine (patch changes) and emits `show-state-changed`.
//!
//! Addressing is 1-based: universes are `1..=512` and DMX addresses `1..=512`. Batch and
//! re-addressing operations work on an "absolute address" (`(universe - 1) * 512 + (address - 1)`)
//! so fixtures that do not fit in the rest of a universe flow to the start of the next one.
//!
//! See `docs/contributing/architecture.md`.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::show::{
    clean_fixture_type, clean_mode, AppState, FixtureGroup, PatchRequest, PatchedFixture,
};

/// Frontend request to patch `count` identical fixtures in one go (camelCase on the wire).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchPatchRequest {
    /// Name of the fixtures; a trailing number is stripped to name the group and fixtures.
    pub name: String,
    /// Fixture type definition (JSON), sanitized with `clean_fixture_type` before storing.
    pub fixture_type: serde_json::Value,
    /// Selected DMX mode (JSON), sanitized with `clean_mode` before storing.
    pub mode: serde_json::Value,
    /// Universe of the first fixture (1-based).
    pub universe: u32,
    /// DMX address of the first fixture within `universe` (1-based, `1..=512`).
    pub start_address: u32,
    /// Number of fixtures to create.
    pub count: u32,
    /// Footprint of one fixture in DMX channels; values below 1 are treated as 1.
    pub channel_count: u32,
}

/// Returns `name` without its trailing numeric word, e.g. `"Par 12"` becomes `"Par"`.
///
/// Used to derive a base name that is then re-suffixed with a sequence number. A name made of a
/// single word (even a number) is returned unchanged.
pub fn extract_standard_name(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() > 1 && parts.last().and_then(|p| p.parse::<u32>().ok()).is_some() {
        parts[..parts.len() - 1].join(" ")
    } else {
        name.to_string()
    }
}

/// Tauri command: patches one fixture into the show and returns it.
///
/// The fixture gets the next free id (`max + 1`) and is assigned to `group_id`, which is not
/// validated. Its footprint is the length of the mode's `channels` array (at least 1). Runs
/// through `AppState::apply_mutation`: one undo step, live engine reset and a
/// `show-state-changed` event on success.
///
/// # Errors
///
/// Returns an error if the universe is outside `1..=512`, the address range does not fit in
/// `1..=512`, it overlaps a fixture already patched on the same universe, or the show/history
/// locks are poisoned. On error the show is left unchanged.
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
            fixture_type: clean_fixture_type(&request.fixture_type),
            mode: clean_mode(&request.mode),
            universe: request.universe,
            address: request.address,
        };

        show.patch.fixtures.push(patched.clone());
        Ok(patched)
    })
}

/// Computes where each fixture of a batch lands and checks it against the existing patch.
///
/// Fixtures are laid out back to back from `universe`/`start_address`; a fixture that would
/// straddle the end of a universe is moved to address 1 of the next one. Returns the planned
/// `(universe, address)` pairs in order.
///
/// # Errors
///
/// Returns an error if a fixture would land beyond universe 512 or overlap an existing fixture.
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

/// Tauri command: patches `request.count` identical fixtures and puts them in a new group.
///
/// A group is created (id `max + 1`) named after the base name; fixtures are named
/// `"<base> <n>"` (1-based), or keep `request.name` verbatim when `count == 1`. The whole batch
/// is a single undo step. Emits `show-state-changed` via `AppState::apply_mutation`.
///
/// # Errors
///
/// Returns an error if the batch exceeds the universe bounds, collides with an existing fixture
/// (see `check_batch_patch_collision`) or the locks are poisoned. On error nothing is created,
/// including the group.
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

        let clean_ft = clean_fixture_type(&request.fixture_type);
        let clean_m = clean_mode(&request.mode);

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
                fixture_type: clean_ft.clone(),
                mode: clean_m.clone(),
                universe,
                address,
            };

            show.patch.fixtures.push(patched.clone());
            created_fixtures.push(patched);
        }

        Ok(created_fixtures)
    })
}

/// Tauri command: removes the fixtures whose ids are in `ids`.
///
/// Unknown ids are ignored; groups are left as they are (even if they become empty). One undo
/// step via `AppState::apply_mutation`.
///
/// # Errors
///
/// Returns an error only if the show/history locks are poisoned.
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

/// Tauri command: renames fixtures.
///
/// With a single id the fixture takes `name` verbatim. With several, they are named
/// `"<base> <n>"` in `ids` order (`n` starts at 1 and only counts ids that exist), where
/// `<base>` is `name` without its trailing number. One undo step via `AppState::apply_mutation`.
///
/// # Errors
///
/// Returns an error only if the show/history locks are poisoned.
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

/// Returns the DMX footprint of a patched fixture: the number of channels in its mode, at least 1.
pub fn get_fixture_channel_count(fixture: &PatchedFixture) -> u64 {
    fixture
        .mode
        .get("channels")
        .and_then(|c| c.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(1)
        .max(1)
}

/// Tauri command: moves fixtures to consecutive addresses starting at `address`.
///
/// The start universe is the one of the first id in `ids`. Fixtures are packed back to back in
/// `ids` order; one that does not fit in the rest of a universe moves to address 1 of the next
/// universe. Unknown ids are skipped and, unlike patching, no bounds or collision check is
/// performed. An empty `ids` is a no-op. One undo step via `AppState::apply_mutation`.
///
/// # Errors
///
/// Returns an error only if the show/history locks are poisoned.
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

/// Tauri command: moves fixtures to `universe`, keeping the address of the first one as start.
///
/// Works like `show_patch_update_address`: fixtures are packed back to back in `ids` order
/// starting at the first fixture's address, overflowing into the following universes; there is
/// no bounds or collision check. An empty `ids` is a no-op. One undo step via
/// `AppState::apply_mutation`.
///
/// # Errors
///
/// Returns an error only if the show/history locks are poisoned.
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

/// Tauri command: toggles membership of the fixtures in the group `group_id`.
///
/// If every listed fixture is already in the group they are all removed from it; otherwise
/// the ones that are missing are added. `group_id == 0` or empty `ids` is a no-op. One undo step
/// via `AppState::apply_mutation`.
///
/// # Errors
///
/// Returns an error only if the show/history locks are poisoned.
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
