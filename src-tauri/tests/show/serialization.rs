use live_dmx_lib::show::{
    clean_fixture_type, clean_mode, clean_patched_fixture, FixtureGroup, PatchGroupsState,
    PatchedFixture, ShowFileContent, ShowIdentityState, ShowOutputState, ShowPatchState,
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
                    name: "All".to_string(),
                },
                FixtureGroup {
                    id: 1,
                    name: "Front Truss".to_string(),
                },
            ],
        },
        output: ShowOutputState::default(),
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
}

#[test]
fn test_show_file_default_version_handling() {
    let json_without_version = r#"{
        "identity": { "displayName": "Default Version Show" },
        "patch": { "fixtures": [] },
        "groups": { "groups": [{ "id": 0, "name": "All" }] }
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

#[test]
fn test_clean_fixture_type_strips_unneeded_data() {
    let bulky_fixture_type = serde_json::json!({
        "name": "Mega Spot 300",
        "shortName": "MS300",
        "fixtureKey": "mega-spot-300",
        "manufacturerKey": "cameo",
        "categories": ["Moving Head", "Spot", "Color Changer"],
        "meta": {
            "authors": ["John Doe", "Jane Smith"],
            "createDate": "2024-01-01",
            "lastModifyDate": "2024-06-01"
        },
        "links": {
            "manual": ["https://example.com/manual.pdf"],
            "productPage": ["https://example.com/product"]
        },
        "physical": {
            "dimensions": [300, 400, 500],
            "weight": 15.5,
            "power": 350
        },
        "matrix": {
            "pixelCount": [1, 1, 1]
        },
        "wheels": {
            "colorWheel": { "slots": [] }
        },
        "availableChannels": {
            "Dimmer": { "defaultValue": 0, "capabilities": [] },
            "Pan": { "defaultValue": 128, "capabilities": [] }
        },
        "modes": [
            { "name": "Standard", "channels": ["Dimmer", "Pan"] },
            { "name": "Extended", "channels": ["Dimmer", "Pan", "Tilt"] }
        ],
        "comment": "A very large comment with lots of text"
    });

    let cleaned = clean_fixture_type(&bulky_fixture_type);
    assert_eq!(cleaned["name"], "Mega Spot 300");
    assert_eq!(cleaned["shortName"], "MS300");
    assert_eq!(cleaned["fixtureKey"], "mega-spot-300");
    assert_eq!(cleaned["manufacturerKey"], "cameo");
    assert!(cleaned.get("categories").is_none());
    assert!(cleaned.get("meta").is_none());
    assert!(cleaned.get("links").is_none());
    assert!(cleaned.get("physical").is_none());
    assert!(cleaned.get("matrix").is_none());
    assert!(cleaned.get("wheels").is_none());
    assert!(cleaned.get("availableChannels").is_none());
    assert!(cleaned.get("modes").is_none());
    assert!(cleaned.get("comment").is_none());
}

#[test]
fn test_clean_mode_simplifies_channels_and_strips_metadata() {
    let bulky_mode = serde_json::json!({
        "name": "Extended 16-Channel",
        "shortName": "16ch",
        "physical": { "weight": 10.0 },
        "rdmPersonalityIndex": 2,
        "channels": [
            "Dimmer",
            { "name": "Pan", "fineChannelAliases": ["Pan fine"] },
            { "channel": "Tilt" },
            "Red",
            "Green",
            "Blue"
        ]
    });

    let cleaned = clean_mode(&bulky_mode);
    assert_eq!(cleaned["name"], "Extended 16-Channel");
    assert_eq!(cleaned["shortName"], "16ch");
    assert!(cleaned.get("physical").is_none());
    assert!(cleaned.get("rdmPersonalityIndex").is_none());

    let channels = cleaned["channels"].as_array().unwrap();
    assert_eq!(channels.len(), 6);
    assert_eq!(channels[0], "Dimmer");
    assert_eq!(channels[1], "Pan");
    assert_eq!(channels[2], "Tilt");
    assert_eq!(channels[3], "Red");
    assert_eq!(channels[4], "Green");
    assert_eq!(channels[5], "Blue");
}

#[test]
fn test_clean_patched_fixture_cleans_entire_fixture() {
    let bulky_fixture = PatchedFixture {
        id: 42,
        groups: vec![0, 3],
        name: "Stage Left Spot".to_string(),
        fixture_type: serde_json::json!({
            "name": "Wash FX",
            "categories": ["Wash"],
            "modes": [{"name": "Standard", "channels": ["Dimmer"]}]
        }),
        mode: serde_json::json!({
            "name": "Standard",
            "channels": ["Dimmer", { "name": "Red" }]
        }),
        universe: 2,
        address: 100,
    };

    let cleaned = clean_patched_fixture(&bulky_fixture);
    assert_eq!(cleaned.id, 42);
    assert_eq!(cleaned.groups, vec![0, 3]);
    assert_eq!(cleaned.name, "Stage Left Spot");
    assert_eq!(cleaned.universe, 2);
    assert_eq!(cleaned.address, 100);
    assert_eq!(cleaned.fixture_type, serde_json::json!({"name": "Wash FX"}));
    assert_eq!(
        cleaned.mode,
        serde_json::json!({
            "name": "Standard",
            "channels": ["Dimmer", "Red"]
        })
    );
}
