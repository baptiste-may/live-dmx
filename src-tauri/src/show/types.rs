//! Show data model: runtime state, history snapshots and the persisted `.ldmx` shape.
//!
//! `ShowData` is the full runtime state sent to the frontend, `ShowSnapshot` is the
//! subset tracked by undo/redo, and `ShowFileContent` is what is written to disk.
//! All types serialize in camelCase. The `clean_*` helpers strip fixture JSON down
//! to the fields that must survive serialization. See `docs/contributing/architecture.md`
//! and `docs/contributing/fixtures.md`.

use serde::{Deserialize, Serialize};

/// Lifecycle status of the current show, driving the load/save UI.
///
/// Serialized as `NOT_LOADED`, `LOADING`, `SAVING` or `LOADED`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShowStatus {
    /// No show is open (initial state, and after unloading).
    #[default]
    NotLoaded,
    /// A show file or MVR import is being read.
    Loading,
    /// A show file is being written.
    Saving,
    /// A show is open and ready.
    Loaded,
}

/// Identity of the show, as stored in the `.ldmx` file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowIdentityState {
    /// Show name shown in the UI.
    pub display_name: String,
}

impl Default for ShowIdentityState {
    fn default() -> Self {
        Self {
            display_name: "My show".to_string(),
        }
    }
}

/// Runtime file/history status of the show. Never persisted in the `.ldmx` file
/// and not part of undo snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowChangesState {
    /// Path of the `.ldmx` file the show was loaded from or last saved to, if any.
    pub path: Option<String>,
    /// `true` when the show differs from the last saved snapshot.
    pub is_dirty: bool,
    /// UI flag: whether the "unsaved changes" confirmation modal is open.
    pub is_unsaved_changes_modal_open: bool,
    /// `true` when the undo stack is not empty.
    pub can_undo: bool,
    /// `true` when the redo stack is not empty.
    pub can_redo: bool,
}

/// Output routing of one DMX universe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UniverseOutputConfig {
    /// Universe number (as used by the patch; the default show starts at 1).
    pub universe: u32,
    /// Target selector: `"none"`, a network protocol (`"artnet"`, `"sacn"`, `"osc"`,
    /// `"kinet"`, `"opc"`) or a physical USB interface identifier.
    pub output: String,
    /// When `false`, the universe is computed but not sent.
    pub enabled: bool,
}

/// Output configuration of the show: one entry per universe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowOutputState {
    /// Configured universes; the commands keep at least one and unique numbers.
    pub universes: Vec<UniverseOutputConfig>,
}

impl Default for ShowOutputState {
    fn default() -> Self {
        Self {
            universes: vec![UniverseOutputConfig {
                universe: 1,
                output: "none".to_string(),
                enabled: true,
            }],
        }
    }
}

use std::collections::HashMap;

/// Mode of a GDTF fixture definition embedded in the show file.
///
/// Optional fields are omitted from the file when `None` and default to `None`
/// when missing, so older files still load.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedGdtfMode {
    /// Mode name, matched case-insensitively against a patched fixture's mode.
    pub name: String,
    /// Short mode name, also accepted when matching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Number of DMX channels the mode occupies.
    pub channel_count: usize,
    /// Channel names, in DMX order.
    pub channels: Vec<String>,
    /// Default DMX value (0-255) of each channel, in channel order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_defaults: Option<Vec<u8>>,
    /// Map of 0-based channel index to the DMX value (0-255) that opens the shutter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutter_open_values: Option<HashMap<usize, u8>>,
    /// Physical pan range in degrees as `[min, max]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan_bounds: Option<[i32; 2]>,
    /// Physical tilt range in degrees as `[min, max]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilt_bounds: Option<[i32; 2]>,
}

/// GDTF fixture definition embedded in the show so it stays portable and
/// usable offline. Stored in [`ShowData::fixture_definitions`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedGdtfDefinition {
    /// Fixture type name.
    pub name: String,
    /// Short fixture type name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Manufacturer name.
    pub manufacturer: String,
    /// Free-form description from the GDTF file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// GDTF `FixtureTypeID` (a GUID string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_type_id: Option<String>,
    /// GDTF-Share revision id of the definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<u32>,
    /// DMX modes offered by the fixture.
    pub modes: Vec<EmbeddedGdtfMode>,
}

/// Undo/redo-tracked subset of [`ShowData`] (everything except `status` and `file`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowSnapshot {
    /// Show identity.
    pub identity: ShowIdentityState,
    /// Patched fixtures.
    pub patch: ShowPatchState,
    /// Fixture groups and their settings.
    pub groups: PatchGroupsState,
    /// Universe output routing (defaults when missing in older files).
    #[serde(default)]
    pub output: ShowOutputState,
    /// Embedded GDTF definitions (defaults to empty when missing in older files).
    #[serde(default)]
    pub fixture_definitions: HashMap<String, EmbeddedGdtfDefinition>,
}

/// Persisted `.ldmx` file content, serialized as pretty-printed JSON.
///
/// Holds the same fields as [`ShowSnapshot`]; runtime-only state (`status`, `file`)
/// is never written. `output` and `fixture_definitions` default when absent so
/// older files still load.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowFileContent {
    /// Show identity.
    pub identity: ShowIdentityState,
    /// Patched fixtures (cleaned with [`clean_patched_fixture`] on save).
    pub patch: ShowPatchState,
    /// Fixture groups and their settings.
    pub groups: PatchGroupsState,
    /// Universe output routing.
    #[serde(default)]
    pub output: ShowOutputState,
    /// Embedded GDTF definitions keyed by `manufacturer@name` in lowercase.
    #[serde(default)]
    pub fixture_definitions: HashMap<String, EmbeddedGdtfDefinition>,
}

/// Preferred colour control for a group. Serialized as the UI label.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ColorPreference {
    /// RGB-style colour mixing (`"Color Mix"`).
    #[default]
    #[serde(rename = "Color Mix")]
    ColorMix,
    /// Colour wheel channel (`"Color Wheel"`).
    #[serde(rename = "Color Wheel")]
    ColorWheel,
    /// Correlated colour temperature (`"CCT"`).
    #[serde(rename = "CCT")]
    Cct,
    /// Colour presets (`"Color presets"`).
    #[serde(rename = "Color presets")]
    ColorPresets,
}

/// How the pixels of multi-pixel fixtures are driven within a group.
///
/// Serialized by variant name; lowercase spellings are accepted as aliases.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PixelPreference {
    /// All pixels act as one.
    #[default]
    #[serde(alias = "single")]
    Single,
    /// Every pixel is controlled separately.
    #[serde(alias = "individual")]
    Individual,
    /// Pixels are split into [`GroupSettings::pixel_group_count`] groups.
    #[serde(alias = "grouped")]
    Grouped,
}

/// Serde default for [`GroupSettings::pan_min`] (degrees).
fn default_pan_min() -> i32 {
    -180
}

/// Serde default for [`GroupSettings::pan_max`] (degrees).
fn default_pan_max() -> i32 {
    180
}

/// Serde default for [`GroupSettings::tilt_min`] (degrees).
fn default_tilt_min() -> i32 {
    -110
}

/// Serde default for [`GroupSettings::tilt_max`] (degrees).
fn default_tilt_max() -> i32 {
    110
}

/// Serde default for [`GroupSettings::pixel_group_count`]; the minimum useful count.
fn default_pixel_group_count() -> u32 {
    2
}

/// Serde default for [`GroupSettings::max_power`] (full power, percent).
fn default_max_power() -> u8 {
    100
}

/// Per-group output settings. Every field has a serde default so partial or
/// older files load. Values are clamped to the group's fixture capabilities by
/// `clamp_group_settings` when updated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupSettings {
    /// Preferred colour control.
    #[serde(default)]
    pub color_preference: ColorPreference,
    /// Preferred pixel control mode.
    #[serde(default)]
    pub pixel_preference: PixelPreference,
    /// Number of pixel groups when [`PixelPreference::Grouped`] (at least 2).
    #[serde(default = "default_pixel_group_count")]
    pub pixel_group_count: u32,
    /// Lower pan limit in degrees.
    #[serde(default = "default_pan_min")]
    pub pan_min: i32,
    /// Upper pan limit in degrees.
    #[serde(default = "default_pan_max")]
    pub pan_max: i32,
    /// Lower tilt limit in degrees.
    #[serde(default = "default_tilt_min")]
    pub tilt_min: i32,
    /// Upper tilt limit in degrees.
    #[serde(default = "default_tilt_max")]
    pub tilt_max: i32,
    /// Invert the pan direction.
    #[serde(default)]
    pub inverse_pan: bool,
    /// Invert the tilt direction.
    #[serde(default)]
    pub inverse_tilt: bool,
    /// Maximum output power as a percentage (0-100).
    #[serde(default = "default_max_power")]
    pub max_power: u8,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            color_preference: ColorPreference::default(),
            pixel_preference: PixelPreference::default(),
            pixel_group_count: default_pixel_group_count(),
            pan_min: default_pan_min(),
            pan_max: default_pan_max(),
            tilt_min: default_tilt_min(),
            tilt_max: default_tilt_max(),
            inverse_pan: false,
            inverse_tilt: false,
            max_power: default_max_power(),
        }
    }
}

/// A named group of fixtures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureGroup {
    /// Group id, unique within the show. `0` is the built-in "All" group.
    pub id: u32,
    /// Group name shown in the UI.
    pub name: String,
}

/// Frontend request to patch one fixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    /// Fixture name.
    pub name: String,
    /// Fixture type JSON (see [`clean_fixture_type`] for the fields kept).
    pub fixture_type: serde_json::Value,
    /// Selected mode JSON (see [`clean_mode`] for the fields kept).
    pub mode: serde_json::Value,
    /// Universe number (1-512).
    pub universe: u32,
    /// Start DMX address within the universe (1-based, 1-512).
    pub address: u32,
}

/// A fixture patched into the show.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PatchedFixture {
    /// Fixture id, unique within the show and allocated as `max + 1`.
    pub id: u32,
    /// Ids of the [`FixtureGroup`]s this fixture belongs to.
    pub groups: Vec<u32>,
    /// Fixture name.
    pub name: String,
    /// Fixture type JSON, kept as free-form JSON and cleaned with [`clean_fixture_type`].
    pub fixture_type: serde_json::Value,
    /// Mode JSON, kept as free-form JSON and cleaned with [`clean_mode`].
    pub mode: serde_json::Value,
    /// Universe number (1-512).
    pub universe: u32,
    /// Start DMX address within the universe (1-based, 1-512).
    pub address: u32,
}

/// Patch of the show: the list of patched fixtures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowPatchState {
    /// Patched fixtures.
    pub fixtures: Vec<PatchedFixture>,
}

/// Fixture groups of the show and their output settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PatchGroupsState {
    /// Groups, always including the built-in "All" group (id `0`).
    pub groups: Vec<FixtureGroup>,
    /// Settings keyed by group id; groups without an entry use defaults.
    #[serde(default)]
    pub settings: HashMap<u32, GroupSettings>,
}

impl Default for PatchGroupsState {
    fn default() -> Self {
        Self {
            groups: vec![FixtureGroup {
                id: 0,
                name: "All".to_string(),
            }],
            settings: HashMap::new(),
        }
    }
}

/// Full runtime state of the show, shared (behind a mutex) between the command
/// handlers and the live output loop, and sent to the frontend as `show-state-changed`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowData {
    /// Lifecycle status (loading/saving/loaded).
    pub status: ShowStatus,
    /// Show identity.
    pub identity: ShowIdentityState,
    /// File path and dirty/undo state; runtime only.
    pub file: ShowChangesState,
    /// Patched fixtures.
    pub patch: ShowPatchState,
    /// Fixture groups and their settings.
    pub groups: PatchGroupsState,
    /// Universe output routing.
    #[serde(default)]
    pub output: ShowOutputState,
    /// Embedded GDTF definitions keyed by `manufacturer@name` in lowercase.
    #[serde(default)]
    pub fixture_definitions: HashMap<String, EmbeddedGdtfDefinition>,
}

impl ShowData {
    /// Clones the undo-tracked part of the show (everything but `status` and `file`).
    pub fn snapshot(&self) -> ShowSnapshot {
        ShowSnapshot {
            identity: self.identity.clone(),
            patch: self.patch.clone(),
            groups: self.groups.clone(),
            output: self.output.clone(),
            fixture_definitions: self.fixture_definitions.clone(),
        }
    }

    /// Overwrites the undo-tracked part of the show with `snapshot`, leaving
    /// `status` and `file` untouched.
    pub fn restore_snapshot(&mut self, snapshot: ShowSnapshot) {
        self.identity = snapshot.identity;
        self.patch = snapshot.patch;
        self.groups = snapshot.groups;
        self.output = snapshot.output;
        self.fixture_definitions = snapshot.fixture_definitions;
    }
}

/// Reduces a fixture type JSON to the fields that must persist: `name`,
/// `shortName`, `fixtureKey`, `manufacturerKey` and non-empty string `categories`.
///
/// Non-object values are returned unchanged.
pub fn clean_fixture_type(val: &serde_json::Value) -> serde_json::Value {
    if !val.is_object() {
        return val.clone();
    }
    let mut map = serde_json::Map::new();
    if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
        map.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
    }
    if let Some(short_name) = val.get("shortName").and_then(|v| v.as_str()) {
        map.insert(
            "shortName".to_string(),
            serde_json::Value::String(short_name.to_string()),
        );
    }
    if let Some(fixture_key) = val.get("fixtureKey").and_then(|v| v.as_str()) {
        map.insert(
            "fixtureKey".to_string(),
            serde_json::Value::String(fixture_key.to_string()),
        );
    }
    if let Some(manufacturer_key) = val.get("manufacturerKey").and_then(|v| v.as_str()) {
        map.insert(
            "manufacturerKey".to_string(),
            serde_json::Value::String(manufacturer_key.to_string()),
        );
    }
    if let Some(categories) = val.get("categories").and_then(|v| v.as_array()) {
        let clean_categories: Vec<serde_json::Value> = categories
            .iter()
            .filter_map(|c| c.as_str().map(|s| serde_json::Value::String(s.to_string())))
            .collect();
        if !clean_categories.is_empty() {
            map.insert(
                "categories".to_string(),
                serde_json::Value::Array(clean_categories),
            );
        }
    }
    serde_json::Value::Object(map)
}

/// Returns `val` as a bounds array only when it is exactly two numbers.
fn clean_bounds(val: Option<&serde_json::Value>) -> Option<serde_json::Value> {
    let arr = val?.as_array()?;
    if arr.len() == 2 && arr[0].is_number() && arr[1].is_number() {
        Some(serde_json::Value::Array(arr.clone()))
    } else {
        None
    }
}

/// Reduces a mode JSON to the fields that must persist: `name`, `shortName`,
/// `channels`, `panBounds`, `tiltBounds`, `channelDefaults` and `shutterOpenValues`.
///
/// Object channels are reduced to their name (or `channel` key, else an empty
/// string), so channel capabilities are not persisted. Non-object values are
/// returned unchanged.
pub fn clean_mode(val: &serde_json::Value) -> serde_json::Value {
    if !val.is_object() {
        return val.clone();
    }
    let mut map = serde_json::Map::new();
    if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
        map.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
    }
    if let Some(short_name) = val.get("shortName").and_then(|v| v.as_str()) {
        map.insert(
            "shortName".to_string(),
            serde_json::Value::String(short_name.to_string()),
        );
    }
    if let Some(channels_val) = val.get("channels").and_then(|v| v.as_array()) {
        let clean_channels: Vec<serde_json::Value> = channels_val
            .iter()
            .map(|ch| {
                if let Some(s) = ch.as_str() {
                    serde_json::Value::String(s.to_string())
                } else if let Some(obj) = ch.as_object() {
                    if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                        serde_json::Value::String(name.to_string())
                    } else if let Some(channel) = obj.get("channel").and_then(|c| c.as_str()) {
                        serde_json::Value::String(channel.to_string())
                    } else {
                        serde_json::Value::String(String::new())
                    }
                } else {
                    serde_json::Value::String(String::new())
                }
            })
            .collect();
        map.insert(
            "channels".to_string(),
            serde_json::Value::Array(clean_channels),
        );
    }
    if let Some(pan_bounds) = clean_bounds(val.get("panBounds")) {
        map.insert("panBounds".to_string(), pan_bounds);
    }
    if let Some(tilt_bounds) = clean_bounds(val.get("tiltBounds")) {
        map.insert("tiltBounds".to_string(), tilt_bounds);
    }
    if let Some(channel_defaults) = val.get("channelDefaults").and_then(|v| v.as_array()) {
        map.insert(
            "channelDefaults".to_string(),
            serde_json::Value::Array(channel_defaults.clone()),
        );
    }
    if let Some(shutter_open_values) = val.get("shutterOpenValues").and_then(|v| v.as_object()) {
        map.insert(
            "shutterOpenValues".to_string(),
            serde_json::Value::Object(shutter_open_values.clone()),
        );
    }
    serde_json::Value::Object(map)
}

/// Returns a copy of `fixture` with its type and mode cleaned for persistence.
pub fn clean_patched_fixture(fixture: &PatchedFixture) -> PatchedFixture {
    PatchedFixture {
        id: fixture.id,
        groups: fixture.groups.clone(),
        name: fixture.name.clone(),
        fixture_type: clean_fixture_type(&fixture.fixture_type),
        mode: clean_mode(&fixture.mode),
        universe: fixture.universe,
        address: fixture.address,
    }
}
