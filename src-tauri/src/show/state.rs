use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

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
    pub selected_group: u32,
}

impl Default for PatchGroupsState {
    fn default() -> Self {
        Self {
            groups: vec![FixtureGroup {
                id: 0,
                name: "All Fixtures".to_string(),
            }],
            selected_group: 0,
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

pub const MAX_UNDO_STACK_SIZE: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct ShowHistory {
    pub undo_stack: Vec<ShowSnapshot>,
    pub redo_stack: Vec<ShowSnapshot>,
    pub saved_snapshot: Option<ShowSnapshot>,
}

impl ShowHistory {
    pub fn reset_with_saved(&mut self, snapshot: ShowSnapshot) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = Some(snapshot);
    }

    pub fn reset_empty(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = None;
    }

    pub fn push_mutation(&mut self, previous_snapshot: ShowSnapshot) {
        self.undo_stack.push(previous_snapshot);
        if self.undo_stack.len() > MAX_UNDO_STACK_SIZE {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn mark_saved(&mut self, current_snapshot: ShowSnapshot) {
        self.saved_snapshot = Some(current_snapshot);
    }

    pub fn is_dirty(&self, current_snapshot: &ShowSnapshot) -> bool {
        match &self.saved_snapshot {
            Some(saved) => saved != current_snapshot,
            None => true,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

pub struct AppState {
    pub show: Mutex<ShowData>,
    pub history: Mutex<ShowHistory>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            show: Mutex::new(ShowData::default()),
            history: Mutex::new(ShowHistory::default()),
        }
    }
}

impl AppState {
    pub fn apply_mutation<F, R>(&self, app: &AppHandle, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut ShowData) -> Result<R, String>,
    {
        let mut show = self.show.lock().map_err(|e| e.to_string())?;
        let mut history = self.history.lock().map_err(|e| e.to_string())?;

        let previous_show = show.clone();
        let prev_snapshot = show.snapshot();
        let result = match f(&mut show) {
            Ok(res) => res,
            Err(err) => {
                *show = previous_show;
                return Err(err);
            }
        };

        let current_snapshot = show.snapshot();
        if prev_snapshot != current_snapshot {
            history.push_mutation(prev_snapshot);
            show.file.is_dirty = history.is_dirty(&current_snapshot);
            show.file.can_undo = history.can_undo();
            show.file.can_redo = history.can_redo();
            emit_show_changed(app, &show);
        }

        Ok(result)
    }
}

pub fn emit_show_changed(app: &AppHandle, show: &ShowData) {
    let _ = app.emit("show-state-changed", show);
}
