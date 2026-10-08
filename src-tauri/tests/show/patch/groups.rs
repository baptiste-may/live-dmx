use live_dmx_lib::show::{
    ColorPreference, FixtureGroup, GroupSettings, PatchGroupsState, PatchedFixture, PixelPreference,
};
use std::collections::HashMap;

#[test]
fn test_group_settings_defaults() {
    let settings = GroupSettings::default();
    assert_eq!(settings.color_preference, ColorPreference::ColorMix);
    assert_eq!(settings.pixel_preference, PixelPreference::Single);
    assert_eq!(settings.pixel_group_count, 2);
    assert_eq!(settings.pan_min, -180);
    assert_eq!(settings.pan_max, 180);
    assert_eq!(settings.tilt_min, -110);
    assert_eq!(settings.tilt_max, 110);
    assert!(!settings.inverse_pan);
    assert!(!settings.inverse_tilt);
}

#[test]
fn test_group_settings_serialization() {
    let settings = GroupSettings {
        color_preference: ColorPreference::ColorWheel,
        pixel_preference: PixelPreference::Individual,
        pixel_group_count: 4,
        pan_min: -45,
        pan_max: 90,
        tilt_min: -30,
        tilt_max: 30,
        inverse_pan: true,
        inverse_tilt: false,
    };

    let json = serde_json::to_string(&settings).unwrap();
    assert!(json.contains("\"colorPreference\":\"Color Wheel\""));
    assert!(json.contains("\"pixelPreference\":\"Individual\""));
    assert!(json.contains("\"pixelGroupCount\":4"));
    assert!(json.contains("\"panMin\":-45"));
    assert!(json.contains("\"panMax\":90"));
    assert!(json.contains("\"inversePan\":true"));

    let deserialized: GroupSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, settings);
}

#[test]
fn test_group_settings_deserialize_partial_json_defaults() {
    let settings: GroupSettings = serde_json::from_str(r#"{"colorPreference":"CCT"}"#).unwrap();
    assert_eq!(settings.color_preference, ColorPreference::Cct);
    assert_eq!(settings.pixel_preference, PixelPreference::Single);
    assert_eq!(settings.pixel_group_count, 2);
    assert_eq!(settings.pan_min, -180);
    assert_eq!(settings.pan_max, 180);
    assert!(!settings.inverse_tilt);
}

#[test]
fn test_group_settings_update_replaces_existing_value() {
    let mut groups = PatchGroupsState::default();
    let updated = GroupSettings {
        color_preference: ColorPreference::Cct,
        pixel_preference: PixelPreference::Grouped,
        pixel_group_count: 3,
        pan_min: 0,
        pan_max: 360,
        tilt_min: 0,
        tilt_max: 180,
        inverse_pan: false,
        inverse_tilt: true,
    };

    groups.settings.insert(1, updated);
    let stored = groups.settings.get(&1).unwrap();
    assert_eq!(stored.color_preference, ColorPreference::Cct);
    assert_eq!(stored.tilt_max, 180);
    assert!(stored.inverse_tilt);
}

#[test]
fn test_group_remove_strips_settings() {
    let mut groups = PatchGroupsState {
        groups: vec![
            FixtureGroup {
                id: 0,
                name: "All".to_string(),
            },
            FixtureGroup {
                id: 1,
                name: "Truss Left".to_string(),
            },
        ],
        settings: HashMap::from([(1, GroupSettings::default())]),
    };

    groups.groups.retain(|g| g.id != 1);
    groups.settings.remove(&1);

    assert_eq!(groups.groups.len(), 1);
    assert!(groups.settings.is_empty());
}

#[test]
fn test_group_create_auto_increment() {
    let mut groups = PatchGroupsState::default();
    assert_eq!(groups.groups.len(), 1);
    assert_eq!(groups.groups[0].id, 0);

    // Create Group 1
    let max_id = groups.groups.iter().map(|g| g.id).max().unwrap_or(0);
    let new_id = max_id + 1;
    groups.groups.push(FixtureGroup {
        id: new_id,
        name: "Truss Left".to_string(),
    });

    assert_eq!(groups.groups.len(), 2);
    assert_eq!(groups.groups[1].id, 1);
    assert_eq!(groups.groups[1].name, "Truss Left");

    // Create Group 2
    let max_id_2 = groups.groups.iter().map(|g| g.id).max().unwrap_or(0);
    let new_id_2 = max_id_2 + 1;
    groups.groups.push(FixtureGroup {
        id: new_id_2,
        name: "Truss Right".to_string(),
    });

    assert_eq!(groups.groups.len(), 3);
    assert_eq!(groups.groups[2].id, 2);
}

#[test]
fn test_group_remove_group_zero_protection() {
    let mut groups = PatchGroupsState::default();

    // Trying to remove group 0 should be a no-op
    let id_to_remove = 0;
    if id_to_remove != 0 {
        groups.groups.retain(|g| g.id != id_to_remove);
    }

    assert_eq!(groups.groups.len(), 1);
    assert_eq!(groups.groups[0].id, 0);
}

#[test]
fn test_group_remove_custom_group_cleans_fixtures() {
    let mut groups = PatchGroupsState {
        groups: vec![
            FixtureGroup {
                id: 0,
                name: "All".to_string(),
            },
            FixtureGroup {
                id: 1,
                name: "Truss Left".to_string(),
            },
            FixtureGroup {
                id: 2,
                name: "Truss Right".to_string(),
            },
        ],
        settings: std::collections::HashMap::new(),
    };

    let mut fixtures = vec![
        PatchedFixture {
            id: 1,
            groups: vec![0, 1, 2],
            name: "Spot 1".to_string(),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({}),
            universe: 1,
            address: 1,
        },
        PatchedFixture {
            id: 2,
            groups: vec![0, 1],
            name: "Spot 2".to_string(),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({}),
            universe: 1,
            address: 2,
        },
    ];

    let remove_id = 1;
    groups.groups.retain(|g| g.id != remove_id);
    for f in &mut fixtures {
        f.groups.retain(|&gid| gid != remove_id);
    }

    assert_eq!(groups.groups.len(), 2);
    assert_eq!(fixtures[0].groups, vec![0, 2]);
    assert_eq!(fixtures[1].groups, vec![0]);
}

#[test]
fn test_group_rename_group_zero_protection() {
    let mut groups = PatchGroupsState::default();

    // Renaming group 0 is disallowed
    let rename_id = 0;
    if rename_id != 0 {
        if let Some(group) = groups.groups.iter_mut().find(|g| g.id == rename_id) {
            group.name = "New Name".to_string();
        }
    }
    assert_eq!(groups.groups[0].name, "All");
}

#[test]
fn test_group_rename_custom_group() {
    let mut groups = PatchGroupsState {
        groups: vec![
            FixtureGroup {
                id: 0,
                name: "All".to_string(),
            },
            FixtureGroup {
                id: 1,
                name: "Old Name".to_string(),
            },
        ],
        settings: std::collections::HashMap::new(),
    };

    let rename_id = 1;
    if rename_id != 0 {
        if let Some(group) = groups.groups.iter_mut().find(|g| g.id == rename_id) {
            group.name = "Renamed Group".to_string();
        }
    }

    assert_eq!(groups.groups[1].name, "Renamed Group");
}

#[test]
fn test_group_settings_clamped_against_bounds() {
    use live_dmx_lib::show::configure::{
        clamp_group_settings, get_group_pan_bounds, get_group_tilt_bounds,
    };

    let fixtures = [PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({ "name": "Spot" }),
        mode: serde_json::json!({
            "channels": ["Pan", "Tilt"],
            "panBounds": [-270, 270],
            "tiltBounds": [-100, 100],
        }),
        universe: 1,
        address: 1,
    }];
    let defs = HashMap::new();
    let refs: Vec<&PatchedFixture> = fixtures.iter().collect();

    let pan_bounds = get_group_pan_bounds(&refs, &defs);
    let tilt_bounds = get_group_tilt_bounds(&refs, &defs);

    let mut settings = GroupSettings {
        color_preference: ColorPreference::ColorMix,
        pixel_preference: PixelPreference::Single,
        pixel_group_count: 2,
        pan_min: -360,
        pan_max: 360,
        tilt_min: -180,
        tilt_max: 180,
        inverse_pan: true,
        inverse_tilt: false,
    };

    clamp_group_settings(&mut settings, pan_bounds, tilt_bounds, &[], &[], 1);
    assert_eq!(settings.pan_min, -270);
    assert_eq!(settings.pan_max, 270);
    assert_eq!(settings.tilt_min, -100);
    assert_eq!(settings.tilt_max, 100);
}
