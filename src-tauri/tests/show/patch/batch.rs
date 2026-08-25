use live_dmx_lib::show::patch::{
    check_batch_patch_collision, extract_standard_name, get_fixture_channel_count,
};
use live_dmx_lib::show::{BatchPatchRequest, PatchedFixture, ShowPatchState};

#[test]
fn test_extract_standard_name_variants() {
    assert_eq!(extract_standard_name("Spot 1"), "Spot");
    assert_eq!(extract_standard_name("Spot"), "Spot");
    assert_eq!(extract_standard_name("Front Light 42"), "Front Light");
    assert_eq!(
        extract_standard_name("Moving Head Pro 100"),
        "Moving Head Pro"
    );
    assert_eq!(extract_standard_name("PAR64"), "PAR64");
    assert_eq!(extract_standard_name("  Wash  Light  3  "), "Wash Light");
    assert_eq!(extract_standard_name("Strobe 0"), "Strobe");
}

#[test]
fn test_get_fixture_channel_count_variations() {
    let fixture_3ch = PatchedFixture {
        id: 1,
        groups: vec![],
        name: "Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": ["dimmer", "pan", "tilt"]
        }),
        universe: 1,
        address: 1,
    };
    assert_eq!(get_fixture_channel_count(&fixture_3ch), 3);

    let fixture_empty_channels = PatchedFixture {
        id: 2,
        groups: vec![],
        name: "Generic".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({
            "channels": []
        }),
        universe: 1,
        address: 1,
    };
    assert_eq!(get_fixture_channel_count(&fixture_empty_channels), 1);

    let fixture_no_mode = PatchedFixture {
        id: 3,
        groups: vec![],
        name: "Unknown".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({}),
        universe: 1,
        address: 1,
    };
    assert_eq!(get_fixture_channel_count(&fixture_no_mode), 1);
}

#[test]
fn test_batch_patch_request_serialization() {
    let req = BatchPatchRequest {
        name: "Spot 1".to_string(),
        fixture_type: serde_json::json!({"name": "Spot"}),
        mode: serde_json::json!({"channels": ["dimmer"]}),
        universe: 2,
        start_address: 10,
        count: 4,
        channel_count: 5,
    };

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"startAddress\":10"));
    assert!(json.contains("\"channelCount\":5"));
    assert!(json.contains("\"universe\":2"));
}

#[test]
fn test_batch_address_calculation_with_universe_rollover() {
    let universe = 1u32;
    let start_address = 510u32;
    let count = 3u32;
    let channel_count = 4u64;

    let mut current_abs_address =
        ((universe.saturating_sub(1)) * 512 + (start_address.saturating_sub(1))) as u64;

    let mut addresses = Vec::new();
    for _ in 0..count {
        let u = ((current_abs_address / 512) + 1) as u32;
        let a = ((current_abs_address % 512) + 1) as u32;
        addresses.push((u, a));
        current_abs_address += channel_count;
    }

    // Fixture 1: Universe 1, Address 510 (channels 510, 511, 512, 513 -> crosses into U2)
    assert_eq!(addresses[0], (1, 510));
    // Fixture 2: Universe 2, Address 2
    assert_eq!(addresses[1], (2, 2));
    // Fixture 3: Universe 2, Address 6
    assert_eq!(addresses[2], (2, 6));
}

#[test]
fn test_batch_patch_naming_single_vs_multiple() {
    let base_name = extract_standard_name("Spot 1");

    // Count = 1 retains given name
    let count_1 = 1;
    let name_single = if count_1 == 1 {
        "Spot 1".to_string()
    } else {
        format!("{} 1", base_name)
    };
    assert_eq!(name_single, "Spot 1");

    // Count = 3 formats with index
    let count_3 = 3;
    let mut names = Vec::new();
    for i in 0..count_3 {
        let fixture_name = if count_3 == 1 {
            "Spot 1".to_string()
        } else {
            format!("{} {}", base_name, i + 1)
        };
        names.push(fixture_name);
    }
    assert_eq!(names, vec!["Spot 1", "Spot 2", "Spot 3"]);
}

#[test]
fn test_patch_fixtures_toggle_group_logic() {
    let mut fixtures = [
        PatchedFixture {
            id: 1,
            groups: vec![1, 2],
            name: "Spot 1".to_string(),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({}),
            universe: 1,
            address: 1,
        },
        PatchedFixture {
            id: 2,
            groups: vec![1],
            name: "Spot 2".to_string(),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({}),
            universe: 1,
            address: 2,
        },
    ];

    let ids = [1, 2];
    let group_id = 2;

    // Not all have group 2 (only fixture 1 does). Toggle should add group 2 to all selected.
    let all_have_group = ids.iter().all(|id| {
        fixtures
            .iter()
            .find(|f| f.id == *id)
            .is_some_and(|f| f.groups.contains(&group_id))
    });
    assert!(!all_have_group);

    for id in &ids {
        if let Some(fixture) = fixtures.iter_mut().find(|f| f.id == *id) {
            if all_have_group {
                fixture.groups.retain(|&g| g != group_id);
            } else if !fixture.groups.contains(&group_id) {
                fixture.groups.push(group_id);
            }
        }
    }

    assert_eq!(fixtures[0].groups, vec![1, 2]);
    assert_eq!(fixtures[1].groups, vec![1, 2]);

    // Now all have group 2. Toggling again should remove group 2 from all selected.
    let all_have_now = ids.iter().all(|id| {
        fixtures
            .iter()
            .find(|f| f.id == *id)
            .is_some_and(|f| f.groups.contains(&group_id))
    });
    assert!(all_have_now);

    for id in &ids {
        if let Some(fixture) = fixtures.iter_mut().find(|f| f.id == *id) {
            if all_have_now {
                fixture.groups.retain(|&g| g != group_id);
            } else if !fixture.groups.contains(&group_id) {
                fixture.groups.push(group_id);
            }
        }
    }

    assert_eq!(fixtures[0].groups, vec![1]);
    assert_eq!(fixtures[1].groups, vec![1]);
}

#[test]
fn test_patch_fixtures_removal() {
    let mut patch = ShowPatchState {
        fixtures: vec![
            PatchedFixture {
                id: 1,
                groups: vec![],
                name: "F1".to_string(),
                fixture_type: serde_json::json!({}),
                mode: serde_json::json!({}),
                universe: 1,
                address: 1,
            },
            PatchedFixture {
                id: 2,
                groups: vec![],
                name: "F2".to_string(),
                fixture_type: serde_json::json!({}),
                mode: serde_json::json!({}),
                universe: 1,
                address: 2,
            },
            PatchedFixture {
                id: 3,
                groups: vec![],
                name: "F3".to_string(),
                fixture_type: serde_json::json!({}),
                mode: serde_json::json!({}),
                universe: 1,
                address: 3,
            },
        ],
    };

    let remove_ids = [1, 3];
    patch.fixtures.retain(|f| !remove_ids.contains(&f.id));

    assert_eq!(patch.fixtures.len(), 1);
    assert_eq!(patch.fixtures[0].id, 2);
}

#[test]
fn test_check_batch_patch_collision_no_overlap() {
    let existing = vec![PatchedFixture {
        id: 1,
        groups: vec![],
        name: "Spot 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["dimmer", "pan", "tilt"] }),
        universe: 1,
        address: 1,
    }];

    let req = BatchPatchRequest {
        name: "Spot 2".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["dimmer", "pan", "tilt"] }),
        universe: 1,
        start_address: 4,
        count: 2,
        channel_count: 3,
    };

    let result = check_batch_patch_collision(&existing, &req);
    assert!(result.is_ok());
    let planned = result.unwrap();
    assert_eq!(planned, vec![(1, 4), (1, 7)]);
}

#[test]
fn test_check_batch_patch_collision_direct_overlap() {
    let existing = vec![PatchedFixture {
        id: 1,
        groups: vec![],
        name: "Spot 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1", "ch2", "ch3", "ch4"] }),
        universe: 1,
        address: 10,
    }];

    let req = BatchPatchRequest {
        name: "Colliding Spot".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1", "ch2"] }),
        universe: 1,
        start_address: 12,
        count: 1,
        channel_count: 2,
    };

    let result = check_batch_patch_collision(&existing, &req);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Address collision"));
}

#[test]
fn test_check_batch_patch_collision_multi_count_second_fixture_overlaps() {
    let existing = vec![PatchedFixture {
        id: 1,
        groups: vec![],
        name: "Existing Fixture".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1", "ch2", "ch3"] }),
        universe: 1,
        address: 20,
    }];

    // Batch starts at 15 with 3 channels each -> Fixture 1 is 15..17 (no collision), Fixture 2 is 18..20 (overlaps at 20!)
    let req = BatchPatchRequest {
        name: "Batch Fixture".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1", "ch2", "ch3"] }),
        universe: 1,
        start_address: 15,
        count: 2,
        channel_count: 3,
    };

    let result = check_batch_patch_collision(&existing, &req);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Address collision"));
}

#[test]
fn test_check_batch_patch_collision_different_universe_no_overlap() {
    let existing = vec![PatchedFixture {
        id: 1,
        groups: vec![],
        name: "Spot U1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1", "ch2"] }),
        universe: 1,
        address: 10,
    }];

    let req = BatchPatchRequest {
        name: "Spot U2".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1", "ch2"] }),
        universe: 2,
        start_address: 10,
        count: 1,
        channel_count: 2,
    };

    let result = check_batch_patch_collision(&existing, &req);
    assert!(result.is_ok());
}

#[test]
fn test_check_batch_patch_collision_universe_limit_exceeded() {
    let existing = vec![];

    let req = BatchPatchRequest {
        name: "Overflow Fixture".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({ "channels": ["ch1"] }),
        universe: 513,
        start_address: 1,
        count: 1,
        channel_count: 1,
    };

    let result = check_batch_patch_collision(&existing, &req);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceeds universe bounds"));
}
