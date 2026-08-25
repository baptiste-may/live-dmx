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
