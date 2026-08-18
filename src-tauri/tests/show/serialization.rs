use live_dmx_lib::show::{
    FixtureGroup, PatchGroupsState, PatchedFixture, ShowFileContent, ShowIdentityState,
    ShowPatchState,
};

#[test]
fn test_show_file_serialization_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test_show.ldmx");

    let content = ShowFileContent {
        version: 1,
        identity: ShowIdentityState {
            display_name: "Concert 2026".to_string(),
        },
        patch: ShowPatchState {
            fixtures: vec![PatchedFixture {
                id: 1,
                groups: vec![0, 1],
                name: "Lead Spot".to_string(),
                fixture_type: serde_json::json!({"name": "Spot Pro"}),
                mode: serde_json::json!({"channels": ["dimmer", "pan", "tilt"]}),
                universe: 1,
                address: 1,
            }],
        },
        groups: PatchGroupsState {
            groups: vec![
                FixtureGroup {
                    id: 0,
                    name: "All Fixtures".to_string(),
                },
                FixtureGroup {
                    id: 1,
                    name: "Front Truss".to_string(),
                },
            ],
            selected_group: 1,
        },
    };

    let json_str = serde_json::to_string_pretty(&content).unwrap();
    std::fs::write(&file_path, &json_str).unwrap();

    let read_str = std::fs::read_to_string(&file_path).unwrap();
    let loaded: ShowFileContent = serde_json::from_str(&read_str).unwrap();

    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.identity.display_name, "Concert 2026");
    assert_eq!(loaded.patch.fixtures.len(), 1);
    assert_eq!(loaded.patch.fixtures[0].name, "Lead Spot");
    assert_eq!(loaded.groups.groups.len(), 2);
    assert_eq!(loaded.groups.selected_group, 1);
}

#[test]
fn test_show_file_default_version_handling() {
    let json_without_version = r#"{
        "identity": { "displayName": "Default Version Show" },
        "patch": { "fixtures": [] },
        "groups": { "groups": [{ "id": 0, "name": "All Fixtures" }], "selectedGroup": 0 }
    }"#;

    let loaded: ShowFileContent = serde_json::from_str(json_without_version).unwrap();
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.identity.display_name, "Default Version Show");
}

#[test]
fn test_show_file_invalid_json() {
    let invalid_json = "{ corrupt json data ... }";
    let result = serde_json::from_str::<ShowFileContent>(invalid_json);
    assert!(result.is_err());
}
