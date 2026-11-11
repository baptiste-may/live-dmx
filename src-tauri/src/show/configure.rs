//! Read-only "configure" view model: group tree, per-group capabilities and clamped settings.
//!
//! The backend is the source of truth for what a group can do (pan/tilt, color modes, pixels):
//! it inspects the channels of the patched fixtures, derives the available options and clamps
//! the stored `GroupSettings` to them, so the frontend never has to duplicate that logic. The
//! computation is pure (`compute_*`) and wrapped by two query commands that only lock the show.
//!
//! See `docs/contributing/architecture.md` (Configure section).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::output::channels::{classify_channel, get_fixture_channel_name};
use crate::output::types::ChannelType;
use crate::show::state::AppState;
use crate::show::types::{
    ColorPreference, EmbeddedGdtfDefinition, FixtureGroup, GroupSettings, PatchedFixture,
    PixelPreference, ShowData,
};

/// One node of the group tree displayed in the configure view (camelCase on the wire).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupRow {
    /// Group id; `0` is the synthetic "All" root.
    pub id: u32,
    /// Display name of the group.
    pub name: String,
    /// Number of distinct fixtures belonging to the group.
    pub fixture_count: usize,
    /// Groups nested under this one (see `compute_group_rows` for how nesting is inferred).
    pub children: Vec<GroupRow>,
}

/// Capabilities and effective settings of one group (camelCase on the wire).
///
/// The `has_*` flags and options are derived from the group's fixtures, and `settings` is
/// already clamped to them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupConfigureInfo {
    /// Group id; `0` is the synthetic "All" group.
    pub id: u32,
    /// Display name of the group.
    pub name: String,
    /// Number of fixtures in the group.
    pub fixture_count: usize,
    /// At least one fixture has a pan axis.
    pub has_pan: bool,
    /// At least one fixture has a tilt axis.
    pub has_tilt: bool,
    /// At least one fixture exposes a color channel (`color_options` is not empty).
    pub has_color: bool,
    /// At least one fixture has more than one pixel.
    pub has_pixels: bool,
    /// Allowed `[min, max]` pan range in degrees (`[-180, 180]` when unknown).
    pub pan_bounds: [i32; 2],
    /// Allowed `[min, max]` tilt range in degrees (`[-110, 110]` when unknown).
    pub tilt_bounds: [i32; 2],
    /// Color modes the group's fixtures support.
    pub color_options: Vec<ColorPreference>,
    /// Pixel modes available; empty when the group has no multi-pixel fixture.
    pub pixel_options: Vec<PixelPreference>,
    /// Largest pixel count among the group's fixtures (at least 1).
    pub max_pixels: usize,
    /// Stored settings (or defaults) after clamping with `clamp_group_settings`.
    pub settings: GroupSettings,
}

/// Everything the configure view needs in one payload (camelCase on the wire).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureOverview {
    /// Group tree, rooted at the "All" row.
    pub rows: Vec<GroupRow>,
    /// Configure info of every group (including `0` for "All"), keyed by group id.
    pub groups: HashMap<u32, GroupConfigureInfo>,
}

/// Tells whether a channel name designates the coarse pan axis.
///
/// Matches `pan` alone or followed by a space, `_` or `-` (case-insensitive); fine/LSB/16-bit
/// channels are excluded so only one channel per axis counts.
pub fn is_pan_channel(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    if lower.contains("fine")
        || lower.contains("lsb")
        || lower.contains("16 bit")
        || lower.contains("16bit")
        || lower.contains("16-bit")
    {
        return false;
    }
    lower == "pan"
        || lower.starts_with("pan ")
        || lower.starts_with("pan_")
        || lower.starts_with("pan-")
}

/// Tells whether a channel name designates the coarse tilt axis.
///
/// Same rules as `is_pan_channel`, for `tilt`.
pub fn is_tilt_channel(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    if lower.contains("fine")
        || lower.contains("lsb")
        || lower.contains("16 bit")
        || lower.contains("16bit")
        || lower.contains("16-bit")
    {
        return false;
    }
    lower == "tilt"
        || lower.starts_with("tilt ")
        || lower.starts_with("tilt_")
        || lower.starts_with("tilt-")
}

/// Parses a JSON `[min, max]` array of integers (degrees) into ordered bounds.
///
/// Swapped values are reordered; returns `None` if the value is not a 2-element array of
/// integers that fit in `i32`.
fn parse_bounds_from_json(val: &serde_json::Value) -> Option<[i32; 2]> {
    let arr = val.as_array()?;
    if arr.len() == 2 {
        let min = i32::try_from(arr[0].as_i64()?).ok()?;
        let max = i32::try_from(arr[1].as_i64()?).ok()?;
        if min <= max {
            return Some([min, max]);
        }
        return Some([max, min]);
    }
    None
}

/// Finds the pan range (degrees) of a patched fixture, if known.
///
/// Lookup order: `panBounds`/`pan_bounds` on the patched mode, then the embedded
/// `definitions`, then the built-in generic fixtures. Fixture and mode names are matched
/// case-insensitively against both `name` and `short_name`.
pub fn get_fixture_pan_bounds(
    fixture: &PatchedFixture,
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
) -> Option<[i32; 2]> {
    if let Some(bounds) = fixture
        .mode
        .get("panBounds")
        .or_else(|| fixture.mode.get("pan_bounds"))
        .and_then(parse_bounds_from_json)
    {
        return Some(bounds);
    }

    let fix_name = fixture
        .fixture_type
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mode_name = fixture
        .mode
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if let Some(def) = definitions.values().find(|d| {
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
            if let Some(pb) = m.pan_bounds {
                return Some(pb);
            }
        }
    }

    let generic_fixtures = crate::fixtures::generic::get_generic_fixtures();
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
            if let Some(pb) = m.pan_bounds {
                return Some(pb);
            }
        }
    }

    None
}

/// Finds the tilt range (degrees) of a patched fixture, if known.
///
/// Uses the same lookup order as `get_fixture_pan_bounds`, with `tiltBounds`/`tilt_bounds`.
pub fn get_fixture_tilt_bounds(
    fixture: &PatchedFixture,
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
) -> Option<[i32; 2]> {
    if let Some(bounds) = fixture
        .mode
        .get("tiltBounds")
        .or_else(|| fixture.mode.get("tilt_bounds"))
        .and_then(parse_bounds_from_json)
    {
        return Some(bounds);
    }

    let fix_name = fixture
        .fixture_type
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mode_name = fixture
        .mode
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if let Some(def) = definitions.values().find(|d| {
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
            if let Some(tb) = m.tilt_bounds {
                return Some(tb);
            }
        }
    }

    let generic_fixtures = crate::fixtures::generic::get_generic_fixtures();
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
            if let Some(tb) = m.tilt_bounds {
                return Some(tb);
            }
        }
    }

    None
}

/// Tells whether a fixture can pan: known pan bounds or a coarse pan channel in its mode.
pub fn fixture_has_pan(
    fixture: &PatchedFixture,
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
) -> bool {
    if get_fixture_pan_bounds(fixture, definitions).is_some() {
        return true;
    }
    let count = crate::output::channels::extract_fixture_channel_count(fixture);
    for i in 0..count {
        if let Some(name) = get_fixture_channel_name(fixture, i) {
            if is_pan_channel(&name) {
                return true;
            }
        }
    }
    false
}

/// Tells whether a fixture can tilt: known tilt bounds or a coarse tilt channel in its mode.
pub fn fixture_has_tilt(
    fixture: &PatchedFixture,
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
) -> bool {
    if get_fixture_tilt_bounds(fixture, definitions).is_some() {
        return true;
    }
    let count = crate::output::channels::extract_fixture_channel_count(fixture);
    for i in 0..count {
        if let Some(name) = get_fixture_channel_name(fixture, i) {
            if is_tilt_channel(&name) {
                return true;
            }
        }
    }
    false
}

/// Returns the union (smallest min, largest max) of the pan ranges of `fixtures`.
///
/// Falls back to `[-180, 180]` degrees when no fixture declares pan bounds.
pub fn get_group_pan_bounds(
    fixtures: &[&PatchedFixture],
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
) -> [i32; 2] {
    let mut min_bound = None;
    let mut max_bound = None;

    for fixture in fixtures {
        if let Some([min, max]) = get_fixture_pan_bounds(fixture, definitions) {
            min_bound = Some(min_bound.map_or(min, |cur: i32| cur.min(min)));
            max_bound = Some(max_bound.map_or(max, |cur: i32| cur.max(max)));
        }
    }

    match (min_bound, max_bound) {
        (Some(min), Some(max)) => [min, max],
        _ => [-180, 180],
    }
}

/// Returns the union (smallest min, largest max) of the tilt ranges of `fixtures`.
///
/// Falls back to `[-110, 110]` degrees when no fixture declares tilt bounds.
pub fn get_group_tilt_bounds(
    fixtures: &[&PatchedFixture],
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
) -> [i32; 2] {
    let mut min_bound = None;
    let mut max_bound = None;

    for fixture in fixtures {
        if let Some([min, max]) = get_fixture_tilt_bounds(fixture, definitions) {
            min_bound = Some(min_bound.map_or(min, |cur: i32| cur.min(min)));
            max_bound = Some(max_bound.map_or(max, |cur: i32| cur.max(max)));
        }
    }

    match (min_bound, max_bound) {
        (Some(min), Some(max)) => [min, max],
        _ => [-110, 110],
    }
}

/// Lists the color modes available to `fixtures`, based on the type of their channels.
///
/// Emits `ColorMix` (any RGB/CMY/white/amber/UV channel), `ColorWheel`, `Cct` and `ColorPresets`
/// (color macro), in that fixed order.
pub fn get_group_color_options(fixtures: &[&PatchedFixture]) -> Vec<ColorPreference> {
    let mut has_mix = false;
    let mut has_wheel = false;
    let mut has_cct = false;
    let mut has_presets = false;

    for fixture in fixtures {
        let count = crate::output::channels::extract_fixture_channel_count(fixture);
        for i in 0..count {
            if let Some(name) = get_fixture_channel_name(fixture, i) {
                let ch_type = classify_channel(&name);
                match ch_type {
                    ChannelType::Red
                    | ChannelType::Green
                    | ChannelType::Blue
                    | ChannelType::White
                    | ChannelType::Amber
                    | ChannelType::Cyan
                    | ChannelType::Magenta
                    | ChannelType::Yellow
                    | ChannelType::UV => has_mix = true,
                    ChannelType::ColorWheel => has_wheel = true,
                    ChannelType::ColorTemperature => has_cct = true,
                    ChannelType::ColorMacro => has_presets = true,
                    _ => {}
                }
            }
        }
    }

    let mut options = Vec::new();
    if has_mix {
        options.push(ColorPreference::ColorMix);
    }
    if has_wheel {
        options.push(ColorPreference::ColorWheel);
    }
    if has_cct {
        options.push(ColorPreference::Cct);
    }
    if has_presets {
        options.push(ColorPreference::ColorPresets);
    }
    options
}

/// Returns the number of pixels of a fixture, as given by its compiled rendering profile.
pub fn get_fixture_pixel_count(fixture: &PatchedFixture) -> usize {
    crate::output::profile::compile_from_patched_fixture(fixture).pixel_count
}

/// Derives the pixel options of `fixtures` as `(has_pixels, options, max_pixels)`.
///
/// With at most 1 pixel there are no options. Up to 3 pixels offer `Single`/`Individual`;
/// beyond that `Grouped` is offered too.
pub fn get_group_pixel_options(
    fixtures: &[&PatchedFixture],
) -> (bool, Vec<PixelPreference>, usize) {
    let mut max_pixels = 1;
    for f in fixtures {
        let count = get_fixture_pixel_count(f);
        if count > max_pixels {
            max_pixels = count;
        }
    }

    if max_pixels <= 1 {
        (false, Vec::new(), 1)
    } else if max_pixels <= 3 {
        (
            true,
            vec![PixelPreference::Single, PixelPreference::Individual],
            max_pixels,
        )
    } else {
        (
            true,
            vec![
                PixelPreference::Single,
                PixelPreference::Individual,
                PixelPreference::Grouped,
            ],
            max_pixels,
        )
    }
}

/// Forces `settings` back inside what the group actually supports.
///
/// Pan/tilt ranges are clamped to the bounds (keeping `min <= max`), color and pixel
/// preferences fall back to the first available option when the stored one is unavailable (or
/// stay untouched when there are no options), `pixel_group_count` is clamped to `2..=max_pixels`
/// (or reset to 2 when `max_pixels < 2`) and `max_power` is capped at 100 (percent).
pub fn clamp_group_settings(
    settings: &mut GroupSettings,
    pan_bounds: [i32; 2],
    tilt_bounds: [i32; 2],
    color_options: &[ColorPreference],
    pixel_options: &[PixelPreference],
    max_pixels: usize,
) {
    settings.pan_min = settings.pan_min.clamp(pan_bounds[0], pan_bounds[1]);
    settings.pan_max = settings.pan_max.clamp(pan_bounds[0], pan_bounds[1]);
    if settings.pan_min > settings.pan_max {
        settings.pan_max = settings.pan_min;
    }

    settings.tilt_min = settings.tilt_min.clamp(tilt_bounds[0], tilt_bounds[1]);
    settings.tilt_max = settings.tilt_max.clamp(tilt_bounds[0], tilt_bounds[1]);
    if settings.tilt_min > settings.tilt_max {
        settings.tilt_max = settings.tilt_min;
    }

    if !color_options.is_empty() && !color_options.contains(&settings.color_preference) {
        settings.color_preference = color_options[0];
    }

    if !pixel_options.is_empty() && !pixel_options.contains(&settings.pixel_preference) {
        settings.pixel_preference = pixel_options[0];
    }
    if max_pixels >= 2 {
        settings.pixel_group_count = settings.pixel_group_count.clamp(2, max_pixels as u32);
    } else {
        settings.pixel_group_count = 2;
    }

    settings.max_power = settings.max_power.min(100);
}

/// Builds the settings of a group that has none stored yet: full pan/tilt range, no
/// inversion, `Single` pixels, 2 pixels per pixel group and 100% max power.
pub fn default_group_settings(
    pan_bounds: [i32; 2],
    tilt_bounds: [i32; 2],
    color_preference: ColorPreference,
) -> GroupSettings {
    GroupSettings {
        color_preference,
        pixel_preference: PixelPreference::Single,
        pixel_group_count: 2,
        pan_min: pan_bounds[0],
        pan_max: pan_bounds[1],
        tilt_min: tilt_bounds[0],
        tilt_max: tilt_bounds[1],
        inverse_pan: false,
        inverse_tilt: false,
        max_power: 100,
    }
}

/// Builds the group tree shown in the configure view.
///
/// The result is a single "All" row (id `0`, counting every fixture) whose children are
/// inferred, since groups carry no explicit parent: a group is nested under the smallest group
/// whose fixture set strictly contains its own. Group `0` entries in `groups` are ignored.
pub fn compute_group_rows(groups: &[FixtureGroup], fixtures: &[PatchedFixture]) -> Vec<GroupRow> {
    let non_zero_groups: Vec<&FixtureGroup> = groups.iter().filter(|g| g.id != 0).collect();

    let group_fixtures: HashMap<u32, HashSet<u32>> = non_zero_groups
        .iter()
        .map(|g| {
            let ids: HashSet<u32> = fixtures
                .iter()
                .filter(|f| f.groups.contains(&g.id))
                .map(|f| f.id)
                .collect();
            (g.id, ids)
        })
        .collect();

    let mut parent_by_id: HashMap<u32, u32> = HashMap::new();

    for group in &non_zero_groups {
        let empty_set = HashSet::new();
        let fix_ids = group_fixtures.get(&group.id).unwrap_or(&empty_set);

        let mut candidates: Vec<&FixtureGroup> = non_zero_groups
            .iter()
            .copied()
            .filter(|candidate| {
                if candidate.id == group.id {
                    return false;
                }
                let cand_ids = group_fixtures.get(&candidate.id).unwrap_or(&empty_set);
                cand_ids.len() > fix_ids.len() && fix_ids.is_subset(cand_ids)
            })
            .collect();

        candidates.sort_by_key(|c| group_fixtures.get(&c.id).map(|s| s.len()).unwrap_or(0));

        if let Some(parent) = candidates.first() {
            parent_by_id.insert(group.id, parent.id);
        }
    }

    /// Recursively builds the rows of the groups whose inferred parent is `parent_id`.
    fn create_children(
        parent_id: u32,
        non_zero_groups: &[&FixtureGroup],
        parent_by_id: &HashMap<u32, u32>,
        group_fixtures: &HashMap<u32, HashSet<u32>>,
    ) -> Vec<GroupRow> {
        non_zero_groups
            .iter()
            .filter(|g| parent_by_id.get(&g.id).copied().unwrap_or(0) == parent_id)
            .map(|g| {
                let fixture_count = group_fixtures.get(&g.id).map(|s| s.len()).unwrap_or(0);
                let children = create_children(g.id, non_zero_groups, parent_by_id, group_fixtures);
                GroupRow {
                    id: g.id,
                    name: g.name.clone(),
                    fixture_count,
                    children,
                }
            })
            .collect()
    }

    vec![GroupRow {
        id: 0,
        name: "All".to_string(),
        fixture_count: fixtures.len(),
        children: create_children(0, &non_zero_groups, &parent_by_id, &group_fixtures),
    }]
}

/// Computes the capabilities and clamped settings of one group.
///
/// `group_id == 0` means every fixture. `current_settings` are the stored settings, if any;
/// otherwise defaults derived from the group's bounds and first color option are used.
pub fn compute_group_configure_info(
    group_id: u32,
    group_name: &str,
    fixtures: &[PatchedFixture],
    definitions: &HashMap<String, EmbeddedGdtfDefinition>,
    current_settings: Option<&GroupSettings>,
) -> GroupConfigureInfo {
    let relevant_fixtures: Vec<&PatchedFixture> = if group_id == 0 {
        fixtures.iter().collect()
    } else {
        fixtures
            .iter()
            .filter(|f| f.groups.contains(&group_id))
            .collect()
    };

    let fixture_count = relevant_fixtures.len();
    let has_pan = relevant_fixtures
        .iter()
        .any(|f| fixture_has_pan(f, definitions));
    let has_tilt = relevant_fixtures
        .iter()
        .any(|f| fixture_has_tilt(f, definitions));
    let color_options = get_group_color_options(&relevant_fixtures);
    let has_color = !color_options.is_empty();
    let pan_bounds = get_group_pan_bounds(&relevant_fixtures, definitions);
    let tilt_bounds = get_group_tilt_bounds(&relevant_fixtures, definitions);

    let (has_pixels, pixel_options, max_pixels) = get_group_pixel_options(&relevant_fixtures);

    let default_color = color_options
        .first()
        .copied()
        .unwrap_or(ColorPreference::ColorMix);
    let mut settings = current_settings
        .copied()
        .unwrap_or_else(|| default_group_settings(pan_bounds, tilt_bounds, default_color));

    clamp_group_settings(
        &mut settings,
        pan_bounds,
        tilt_bounds,
        &color_options,
        &pixel_options,
        max_pixels,
    );

    GroupConfigureInfo {
        id: group_id,
        name: group_name.to_string(),
        fixture_count,
        has_pan,
        has_tilt,
        has_color,
        has_pixels,
        pan_bounds,
        tilt_bounds,
        color_options,
        pixel_options,
        max_pixels,
        settings,
    }
}

/// Computes the full configure payload (tree plus info for "All" and every group) of a show.
pub fn compute_configure_overview(show: &ShowData) -> ConfigureOverview {
    let rows = compute_group_rows(&show.groups.groups, &show.patch.fixtures);
    let mut groups = HashMap::new();

    let all_info = compute_group_configure_info(
        0,
        "All",
        &show.patch.fixtures,
        &show.fixture_definitions,
        show.groups.settings.get(&0),
    );
    groups.insert(0, all_info);

    for group in &show.groups.groups {
        if group.id == 0 {
            continue;
        }
        let info = compute_group_configure_info(
            group.id,
            &group.name,
            &show.patch.fixtures,
            &show.fixture_definitions,
            show.groups.settings.get(&group.id),
        );
        groups.insert(group.id, info);
    }

    ConfigureOverview { rows, groups }
}

/// Tauri command: returns the configure overview of the current show.
///
/// Read-only: it neither mutates `AppState` nor emits events, and is not an undo step.
///
/// # Errors
///
/// Returns an error if the show lock is poisoned.
#[tauri::command]
pub fn show_configure_get_overview(
    state: State<'_, AppState>,
) -> Result<ConfigureOverview, String> {
    let show = state.show.lock().map_err(|e| e.to_string())?;
    Ok(compute_configure_overview(&show))
}

/// Tauri command: returns the configure info of a single group.
///
/// Group `0` is always "All"; `Ok(None)` is returned for an unknown group id. Read-only: no
/// event and no undo step.
///
/// # Errors
///
/// Returns an error if the show lock is poisoned.
#[tauri::command]
pub fn show_configure_get_group(
    state: State<'_, AppState>,
    group_id: u32,
) -> Result<Option<GroupConfigureInfo>, String> {
    let show = state.show.lock().map_err(|e| e.to_string())?;
    let name = if group_id == 0 {
        Some("All".to_string())
    } else {
        show.groups
            .groups
            .iter()
            .find(|g| g.id == group_id)
            .map(|g| g.name.clone())
    };
    let Some(name) = name else {
        return Ok(None);
    };
    Ok(Some(compute_group_configure_info(
        group_id,
        &name,
        &show.patch.fixtures,
        &show.fixture_definitions,
        show.groups.settings.get(&group_id),
    )))
}
