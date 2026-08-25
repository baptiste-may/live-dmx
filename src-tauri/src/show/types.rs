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
pub struct ShowSnapshot {
    pub identity: ShowIdentityState,
    pub patch: ShowPatchState,
    pub groups: PatchGroupsState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowFileContent {
    #[serde(default = "default_file_version")]
    pub version: u32,
    pub identity: ShowIdentityState,
    pub patch: ShowPatchState,
    pub groups: PatchGroupsState,
}

fn default_file_version() -> u32 {
    1
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
}

impl Default for PatchGroupsState {
    fn default() -> Self {
        Self {
            groups: vec![FixtureGroup {
                id: 0,
                name: "All".to_string(),
            }],
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
}

impl ShowData {
    pub fn snapshot(&self) -> ShowSnapshot {
        ShowSnapshot {
            identity: self.identity.clone(),
            patch: self.patch.clone(),
            groups: self.groups.clone(),
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: ShowSnapshot) {
        self.identity = snapshot.identity;
        self.patch = snapshot.patch;
        self.groups = snapshot.groups;
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
    serde_json::Value::Object(map)
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
