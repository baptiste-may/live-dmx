use std::collections::HashMap;

use live_dmx_lib::show::{
    clamp_group_settings, compute_configure_overview, compute_group_configure_info,
    compute_group_rows, default_group_settings, get_group_color_options, get_group_pan_bounds,
    get_group_tilt_bounds, is_pan_channel, is_tilt_channel, ColorPreference, FixtureGroup,
    GroupSettings, PatchGroupsState, PatchedFixture, ShowData, ShowPatchState,
};

#[test]
fn test_is_pan_and_tilt_channel() {
    assert!(is_pan_channel("Pan"));
    assert!(is_pan_channel("pan"));
    assert!(is_pan_channel("Pan 1"));
    assert!(is_pan_channel("Pan_Continuous"));
    assert!(!is_pan_channel("Pan fine"));
    assert!(!is_pan_channel("Pan (16 bit)"));
    assert!(!is_pan_channel("Pan LSB"));
    assert!(!is_pan_channel("Panel"));

    assert!(is_tilt_channel("Tilt"));
    assert!(is_tilt_channel("tilt"));
    assert!(is_tilt_channel("Tilt 1"));
    assert!(is_tilt_channel("Tilt-Continuous"));
    assert!(!is_tilt_channel("Tilt fine"));
    assert!(!is_tilt_channel("Tilt (16 bit)"));
    assert!(!is_tilt_channel("Tilt LSB"));
}

#[test]
fn test_compute_group_rows_empty_and_root() {
    let groups = vec![FixtureGroup {
        id: 0,
        name: "All".to_string(),
    }];
    let fixtures = vec![];

    let rows = compute_group_rows(&groups, &fixtures);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 0);
    assert_eq!(rows[0].name, "All");
    assert_eq!(rows[0].fixture_count, 0);
    assert!(rows[0].children.is_empty());
}

#[test]
fn test_compute_group_rows_nested_hierarchy() {
    let groups = vec![
        FixtureGroup {
            id: 0,
            name: "All".to_string(),
        },
        FixtureGroup {
            id: 1,
            name: "All fixtures".to_string(),
        },
        FixtureGroup {
            id: 2,
            name: "Moving heads".to_string(),
        },
    ];

    let fixtures = vec![
        PatchedFixture {
            id: 1,
            groups: vec![1, 2],
            name: "Head 1".to_string(),
            fixture_type: serde_json::json!({ "name": "Head" }),
            mode: serde_json::json!({ "name": "Mode", "channels": ["Pan", "Tilt"] }),
            universe: 1,
            address: 1,
        },
        PatchedFixture {
            id: 2,
            groups: vec![1],
            name: "Head 2".to_string(),
            fixture_type: serde_json::json!({ "name": "Head" }),
            mode: serde_json::json!({ "name": "Mode", "channels": ["Pan", "Tilt"] }),
            universe: 1,
            address: 10,
        },
    ];

    let rows = compute_group_rows(&groups, &fixtures);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 0);
    assert_eq!(rows[0].fixture_count, 2);
    assert_eq!(rows[0].children.len(), 1);

    let child = &rows[0].children[0];
    assert_eq!(child.id, 1);
    assert_eq!(child.name, "All fixtures");
    assert_eq!(child.fixture_count, 2);
    assert_eq!(child.children.len(), 1);

    let grandchild = &child.children[0];
    assert_eq!(grandchild.id, 2);
    assert_eq!(grandchild.name, "Moving heads");
    assert_eq!(grandchild.fixture_count, 1);
    assert!(grandchild.children.is_empty());
}

#[test]
fn test_group_color_options_detection() {
    let fix_mix = PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "RGB".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["Red", "Green", "Blue", "White"] }),
        universe: 1,
        address: 1,
    };
    let fix_wheel = PatchedFixture {
        id: 2,
        groups: vec![2],
        name: "Wheel".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["Color Wheel"] }),
        universe: 1,
        address: 10,
    };
    let fix_cct = PatchedFixture {
        id: 3,
        groups: vec![3],
        name: "CCT".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["CCT"] }),
        universe: 1,
        address: 20,
    };
    let fix_macro = PatchedFixture {
        id: 4,
        groups: vec![4],
        name: "Preset".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["Color Macro"] }),
        universe: 1,
        address: 30,
    };

    assert_eq!(
        get_group_color_options(&[&fix_mix]),
        vec![ColorPreference::ColorMix]
    );
    assert_eq!(
        get_group_color_options(&[&fix_wheel]),
        vec![ColorPreference::ColorWheel]
    );
    assert_eq!(
        get_group_color_options(&[&fix_cct]),
        vec![ColorPreference::Cct]
    );
    assert_eq!(
        get_group_color_options(&[&fix_macro]),
        vec![ColorPreference::ColorPresets]
    );
    assert_eq!(
        get_group_color_options(&[&fix_mix, &fix_wheel, &fix_cct, &fix_macro]),
        vec![
            ColorPreference::ColorMix,
            ColorPreference::ColorWheel,
            ColorPreference::Cct,
            ColorPreference::ColorPresets
        ]
    );
}

#[test]
fn test_group_pan_tilt_bounds_calculation() {
    let defs = HashMap::new();
    let fixtures = vec![
        PatchedFixture {
            id: 1,
            groups: vec![1],
            name: "Spot A".to_string(),
            fixture_type: serde_json::json!({ "name": "Spot A" }),
            mode: serde_json::json!({
                "channels": ["Pan", "Tilt"],
                "panBounds": [-270, 270],
                "tiltBounds": [-135, 135]
            }),
            universe: 1,
            address: 1,
        },
        PatchedFixture {
            id: 2,
            groups: vec![1],
            name: "Spot B".to_string(),
            fixture_type: serde_json::json!({ "name": "Spot B" }),
            mode: serde_json::json!({
                "channels": ["Pan", "Tilt"],
                "panBounds": [-360, 360],
                "tiltBounds": [-100, 100]
            }),
            universe: 1,
            address: 10,
        },
    ];

    let refs: Vec<&PatchedFixture> = fixtures.iter().collect();
    let pan_bounds = get_group_pan_bounds(&refs, &defs);
    let tilt_bounds = get_group_tilt_bounds(&refs, &defs);

    assert_eq!(pan_bounds, [-360, 360]);
    assert_eq!(tilt_bounds, [-135, 135]);

    // Empty fixtures fallback
    let empty_refs: Vec<&PatchedFixture> = vec![];
    assert_eq!(get_group_pan_bounds(&empty_refs, &defs), [-180, 180]);
    assert_eq!(get_group_tilt_bounds(&empty_refs, &defs), [-110, 110]);
}

#[test]
fn test_clamp_group_settings() {
    let mut settings = GroupSettings {
        color_preference: ColorPreference::ColorPresets,
        pan_min: -500,
        pan_max: 500,
        tilt_min: 200,
        tilt_max: -200,
        inverse_pan: false,
        inverse_tilt: false,
    };

    let pan_bounds = [-270, 270];
    let tilt_bounds = [-135, 135];
    let color_options = vec![ColorPreference::ColorMix, ColorPreference::ColorWheel];

    clamp_group_settings(&mut settings, pan_bounds, tilt_bounds, &color_options);

    assert_eq!(settings.pan_min, -270);
    assert_eq!(settings.pan_max, 270);
    assert_eq!(settings.tilt_min, 135);
    assert_eq!(settings.tilt_max, 135);
    // Invalid color preference was clamped to first available option
    assert_eq!(settings.color_preference, ColorPreference::ColorMix);
}

#[test]
fn test_default_group_settings() {
    let settings = default_group_settings([-270, 270], [-135, 135], ColorPreference::ColorWheel);
    assert_eq!(settings.pan_min, -270);
    assert_eq!(settings.pan_max, 270);
    assert_eq!(settings.tilt_min, -135);
    assert_eq!(settings.tilt_max, 135);
    assert_eq!(settings.color_preference, ColorPreference::ColorWheel);
    assert!(!settings.inverse_pan);
    assert!(!settings.inverse_tilt);
}

#[test]
fn test_compute_configure_overview_full() {
    let mut show = ShowData::default();
    show.groups = PatchGroupsState {
        groups: vec![
            FixtureGroup {
                id: 0,
                name: "All".to_string(),
            },
            FixtureGroup {
                id: 1,
                name: "Moving Heads".to_string(),
            },
            FixtureGroup {
                id: 2,
                name: "Static RGB".to_string(),
            },
        ],
        settings: HashMap::from([(
            1,
            GroupSettings {
                color_preference: ColorPreference::ColorWheel,
                pan_min: -90,
                pan_max: 90,
                tilt_min: -45,
                tilt_max: 45,
                inverse_pan: true,
                inverse_tilt: false,
            },
        )]),
    };

    show.patch = ShowPatchState {
        fixtures: vec![
            PatchedFixture {
                id: 1,
                groups: vec![1],
                name: "Spot 1".to_string(),
                fixture_type: serde_json::json!({ "name": "Spot" }),
                mode: serde_json::json!({
                    "channels": ["Pan", "Tilt", "Color Wheel"],
                    "panBounds": [-270, 270],
                    "tiltBounds": [-135, 135]
                }),
                universe: 1,
                address: 1,
            },
            PatchedFixture {
                id: 2,
                groups: vec![2],
                name: "PAR 1".to_string(),
                fixture_type: serde_json::json!({ "name": "PAR" }),
                mode: serde_json::json!({
                    "channels": ["Red", "Green", "Blue"],
                }),
                universe: 1,
                address: 20,
            },
        ],
    };

    let overview = compute_configure_overview(&show);
    assert_eq!(overview.rows.len(), 1);
    assert_eq!(overview.rows[0].id, 0);
    assert_eq!(overview.rows[0].fixture_count, 2);

    let all_info = overview.groups.get(&0).unwrap();
    assert_eq!(all_info.id, 0);
    assert_eq!(all_info.fixture_count, 2);
    assert!(all_info.has_pan);
    assert!(all_info.has_tilt);
    assert!(all_info.has_color);
    assert_eq!(all_info.pan_bounds, [-270, 270]);
    assert_eq!(all_info.tilt_bounds, [-135, 135]);
    assert_eq!(
        all_info.color_options,
        vec![ColorPreference::ColorMix, ColorPreference::ColorWheel]
    );

    let head_info = overview.groups.get(&1).unwrap();
    assert_eq!(head_info.id, 1);
    assert_eq!(head_info.fixture_count, 1);
    assert!(head_info.has_pan);
    assert!(head_info.has_tilt);
    assert!(head_info.has_color);
    assert_eq!(head_info.color_options, vec![ColorPreference::ColorWheel]);
    assert_eq!(head_info.settings.pan_min, -90);
    assert_eq!(head_info.settings.pan_max, 90);
    assert!(head_info.settings.inverse_pan);

    let par_info = overview.groups.get(&2).unwrap();
    assert_eq!(par_info.id, 2);
    assert_eq!(par_info.fixture_count, 1);
    assert!(!par_info.has_pan);
    assert!(!par_info.has_tilt);
    assert!(par_info.has_color);
    assert_eq!(par_info.color_options, vec![ColorPreference::ColorMix]);
}

#[test]
fn test_compute_group_configure_info() {
    let defs = HashMap::new();
    let fixtures = vec![PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({ "name": "Spot" }),
        mode: serde_json::json!({
            "channels": ["Pan", "Tilt", "Red", "Green", "Blue"],
            "panBounds": [-270, 270],
            "tiltBounds": [-135, 135],
        }),
        universe: 1,
        address: 1,
    }];

    let info = compute_group_configure_info(1, "Moving Heads", &fixtures, &defs, None);
    assert_eq!(info.id, 1);
    assert_eq!(info.name, "Moving Heads");
    assert_eq!(info.fixture_count, 1);
    assert!(info.has_pan);
    assert!(info.has_tilt);
    assert!(info.has_color);
    assert_eq!(info.pan_bounds, [-270, 270]);
    assert_eq!(info.tilt_bounds, [-135, 135]);
    assert_eq!(info.color_options, vec![ColorPreference::ColorMix]);
    assert_eq!(info.settings.pan_min, -270);
    assert_eq!(info.settings.pan_max, 270);
}
