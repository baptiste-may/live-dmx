use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShowStatus {
    #[default]
    NotLoaded,
    Loading,
    Saving,
    Loaded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowIdentityState {
    pub display_name: String,
}

impl Default for ShowIdentityState {
    fn default() -> Self {
        Self {
            display_name: "My show".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowChangesState {
    pub path: Option<String>,
    pub is_dirty: bool,
    pub is_unsaved_changes_modal_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UniverseOutputConfig {
    pub universe: u32,
    pub output: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowOutputState {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedGdtfMode {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    pub channel_count: usize,
    pub channels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_defaults: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutter_open_values: Option<HashMap<usize, u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan_bounds: Option<[i32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilt_bounds: Option<[i32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedGdtfDefinition {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    pub manufacturer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_type_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rid: Option<u32>,
    pub modes: Vec<EmbeddedGdtfMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowSnapshot {
    pub identity: ShowIdentityState,
    pub patch: ShowPatchState,
    pub groups: PatchGroupsState,
    #[serde(default)]
    pub output: ShowOutputState,
    #[serde(default)]
    pub fixture_definitions: HashMap<String, EmbeddedGdtfDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowFileContent {
    pub identity: ShowIdentityState,
    pub patch: ShowPatchState,
    pub groups: PatchGroupsState,
    #[serde(default)]
    pub output: ShowOutputState,
    #[serde(default)]
    pub fixture_definitions: HashMap<String, EmbeddedGdtfDefinition>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ColorPreference {
    #[default]
    #[serde(rename = "Color Mix")]
    ColorMix,
    #[serde(rename = "Color Wheel")]
    ColorWheel,
    #[serde(rename = "CCT")]
    Cct,
    #[serde(rename = "Color presets")]
    ColorPresets,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PixelPreference {
    #[default]
    #[serde(alias = "single")]
    Single,
    #[serde(alias = "individual")]
    Individual,
    #[serde(alias = "grouped")]
    Grouped,
}

fn default_pan_min() -> i32 {
    -180
}

fn default_pan_max() -> i32 {
    180
}

fn default_tilt_min() -> i32 {
    -110
}

fn default_tilt_max() -> i32 {
    110
}

fn default_pixel_group_count() -> u32 {
    2
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupSettings {
    #[serde(default)]
    pub color_preference: ColorPreference,
    #[serde(default)]
    pub pixel_preference: PixelPreference,
    #[serde(default = "default_pixel_group_count")]
    pub pixel_group_count: u32,
    #[serde(default = "default_pan_min")]
    pub pan_min: i32,
    #[serde(default = "default_pan_max")]
    pub pan_max: i32,
    #[serde(default = "default_tilt_min")]
    pub tilt_min: i32,
    #[serde(default = "default_tilt_max")]
    pub tilt_max: i32,
    #[serde(default)]
    pub inverse_pan: bool,
    #[serde(default)]
    pub inverse_tilt: bool,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureGroup {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    pub name: String,
    pub fixture_type: serde_json::Value,
    pub mode: serde_json::Value,
    pub universe: u32,
    pub address: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PatchedFixture {
    pub id: u32,
    pub groups: Vec<u32>,
    pub name: String,
    pub fixture_type: serde_json::Value,
    pub mode: serde_json::Value,
    pub universe: u32,
    pub address: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowPatchState {
    pub fixtures: Vec<PatchedFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PatchGroupsState {
    pub groups: Vec<FixtureGroup>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowData {
    pub status: ShowStatus,
    pub identity: ShowIdentityState,
    pub file: ShowChangesState,
    pub patch: ShowPatchState,
    pub groups: PatchGroupsState,
    #[serde(default)]
    pub output: ShowOutputState,
    #[serde(default)]
    pub fixture_definitions: HashMap<String, EmbeddedGdtfDefinition>,
}

impl ShowData {
    pub fn snapshot(&self) -> ShowSnapshot {
        ShowSnapshot {
            identity: self.identity.clone(),
            patch: self.patch.clone(),
            groups: self.groups.clone(),
            output: self.output.clone(),
            fixture_definitions: self.fixture_definitions.clone(),
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: ShowSnapshot) {
        self.identity = snapshot.identity;
        self.patch = snapshot.patch;
        self.groups = snapshot.groups;
        self.output = snapshot.output;
        self.fixture_definitions = snapshot.fixture_definitions;
    }
}

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

fn clean_bounds(val: Option<&serde_json::Value>) -> Option<serde_json::Value> {
    let arr = val?.as_array()?;
    if arr.len() == 2 && arr[0].is_number() && arr[1].is_number() {
        Some(serde_json::Value::Array(arr.clone()))
    } else {
        None
    }
}

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
