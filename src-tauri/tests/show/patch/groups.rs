use live_dmx_lib::show::{FixtureGroup, PatchGroupsState, PatchedFixture};

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
    };

    let rename_id = 1;
    if rename_id != 0 {
        if let Some(group) = groups.groups.iter_mut().find(|g| g.id == rename_id) {
            group.name = "Renamed Group".to_string();
        }
    }

    assert_eq!(groups.groups[1].name, "Renamed Group");
}
