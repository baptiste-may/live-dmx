use live_dmx_lib::show::{
    AppState, FixtureGroup, PatchGroupsState, PatchRequest, PatchedFixture, ShowChangesState,
    ShowData, ShowIdentityState, ShowPatchState, ShowSnapshot, ShowStatus,
};

#[test]
fn test_show_status_serialization() {
    let status = ShowStatus::NotLoaded;
    assert_eq!(status, ShowStatus::default());

    let json = serde_json::to_string(&ShowStatus::NotLoaded).unwrap();
    assert_eq!(json, "\"NOT_LOADED\"");

    let loaded_status: ShowStatus = serde_json::from_str("\"LOADED\"").unwrap();
    assert_eq!(loaded_status, ShowStatus::Loaded);
}

#[test]
fn test_show_identity_state_defaults() {
    let identity = ShowIdentityState::default();
    assert_eq!(identity.display_name, "My show");

    let json = serde_json::to_string(&identity).unwrap();
    assert!(json.contains("\"displayName\":\"My show\""));
}

#[test]
fn test_show_changes_state_defaults() {
    let changes = ShowChangesState::default();
    assert_eq!(changes.path, None);
    assert!(!changes.is_dirty);
    assert!(!changes.is_unsaved_changes_modal_open);
    assert!(!changes.can_undo);
    assert!(!changes.can_redo);
}

#[test]
fn test_patch_groups_state_defaults() {
    let groups = PatchGroupsState::default();
    assert_eq!(groups.groups.len(), 1);
    assert_eq!(groups.groups[0].id, 0);
    assert_eq!(groups.groups[0].name, "All");
}

#[test]
fn test_patched_fixture_and_patch_request_serialization() {
    let request = PatchRequest {
        name: "Moving Head".to_string(),
        fixture_type: serde_json::json!({"name": "MH 1"}),
        mode: serde_json::json!({"name": "16ch"}),
        universe: 1,
        address: 10,
    };

    let req_json = serde_json::to_string(&request).unwrap();
    assert!(req_json.contains("\"fixtureType\""));
    assert!(req_json.contains("\"universe\":1"));
    assert!(req_json.contains("\"address\":10"));

    let patched = PatchedFixture {
        id: 1,
        groups: vec![0, 2],
        name: "Moving Head".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({}),
        universe: 1,
        address: 10,
    };

    let patched_json = serde_json::to_string(&patched).unwrap();
    let deserialized: PatchedFixture = serde_json::from_str(&patched_json).unwrap();
    assert_eq!(deserialized, patched);
}

#[test]
fn test_show_data_snapshot_and_restore() {
    let mut show = ShowData {
        status: ShowStatus::Loaded,
        identity: ShowIdentityState {
            display_name: "Original Show".to_string(),
        },
        file: ShowChangesState {
            path: Some("/tmp/show.ldmx".to_string()),
            is_dirty: true,
            is_unsaved_changes_modal_open: false,
            can_undo: true,
            can_redo: false,
        },
        patch: ShowPatchState {
            fixtures: vec![PatchedFixture {
                id: 1,
                groups: vec![0],
                name: "Spot".to_string(),
                fixture_type: serde_json::json!({}),
                mode: serde_json::json!({}),
                universe: 1,
                address: 1,
            }],
        },
        groups: PatchGroupsState {
            groups: vec![
                FixtureGroup {
                    id: 0,
                    name: "All".to_string(),
                },
                FixtureGroup {
                    id: 1,
                    name: "Front".to_string(),
                },
            ],
        },
    };

    let snapshot = show.snapshot();
    assert_eq!(snapshot.identity.display_name, "Original Show");
    assert_eq!(snapshot.patch.fixtures.len(), 1);
    assert_eq!(snapshot.groups.groups.len(), 2);

    let new_snapshot = ShowSnapshot {
        identity: ShowIdentityState {
            display_name: "Restored Show".to_string(),
        },
        patch: ShowPatchState::default(),
        groups: PatchGroupsState::default(),
    };

    show.restore_snapshot(new_snapshot);
    assert_eq!(show.identity.display_name, "Restored Show");
    assert_eq!(show.patch.fixtures.len(), 0);
    assert_eq!(show.groups.groups.len(), 1);
    // Note: status and file are untouched by restore_snapshot
    assert_eq!(show.status, ShowStatus::Loaded);
    assert_eq!(show.file.path.as_deref(), Some("/tmp/show.ldmx"));
}

#[test]
fn test_app_state_default() {
    let app_state = AppState::default();
    let show = app_state.show.lock().unwrap();
    assert_eq!(show.identity.display_name, "My show");
    assert_eq!(show.status, ShowStatus::NotLoaded);

    let history = app_state.history.lock().unwrap();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_app_state_mutate_show_resets_live_engine_on_patch_change() {
    let app_state = AppState::default();

    // Set some live engine state
    {
        let mut live = app_state.output.engine.lock().unwrap();
        live.set_group_master(1, 50);
        live.set_button("btn_1".to_string(), true);
    }

    // Mutate patch
    let res = app_state.mutate_show(|show| {
        show.patch.fixtures.push(PatchedFixture {
            id: 1,
            groups: vec![1],
            name: "New Fixture".to_string(),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({}),
            universe: 1,
            address: 1,
        });
        Ok(())
    });
    assert!(res.is_ok());

    // Live state should be reset
    let live = app_state.output.engine.lock().unwrap();
    assert_eq!(live.get_group_master(1), 100);
    assert_eq!(live.button_states.get("btn_1"), None);
}

#[test]
fn test_app_state_mutate_show_resets_live_engine_on_group_change() {
    let app_state = AppState::default();

    // Set some live engine state
    {
        let mut live = app_state.output.engine.lock().unwrap();
        live.set_group_master(1, 75);
    }

    // Mutate groups
    let res = app_state.mutate_show(|show| {
        show.groups.groups.push(FixtureGroup {
            id: 1,
            name: "Group 1".to_string(),
        });
        Ok(())
    });
    assert!(res.is_ok());

    // Live state should be reset
    let live = app_state.output.engine.lock().unwrap();
    assert_eq!(live.get_group_master(1), 100);
}

#[test]
fn test_app_state_mutate_show_preserves_live_engine_on_identity_only_change() {
    let app_state = AppState::default();

    // Set some live engine state
    {
        let mut live = app_state.output.engine.lock().unwrap();
        live.set_group_master(1, 42);
        live.set_button("btn_test".to_string(), true);
    }

    // Mutate identity only
    let res = app_state.mutate_show(|show| {
        show.identity.display_name = "Renamed Show".to_string();
        Ok(())
    });
    assert!(res.is_ok());

    // Live state should be preserved
    let live = app_state.output.engine.lock().unwrap();
    assert_eq!(live.get_group_master(1), 42);
    assert_eq!(live.button_states.get("btn_test"), Some(&true));
}
