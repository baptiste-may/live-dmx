use live_dmx_lib::output::{hsv_to_rgb, resolve_color_rgb, LiveEngine};
use live_dmx_lib::show::{PatchedFixture, ShowData};

#[test]
fn test_live_engine_group_color_calculation() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "LED Wash".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue", "White"]}),
        universe: 1,
        address: 1,
    });

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(1, Some("full".to_string()), None, false);

    // Set Group 1 color to Red
    engine.set_group_color(1, Some("red".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255); // Dimmer
    assert_eq!(buf[1], 255); // Red
    assert_eq!(buf[2], 0); // Green
    assert_eq!(buf[3], 0); // Blue
    assert_eq!(buf[4], 0); // White

    // Set Group 1 color to Cyan (#00FFFF)
    engine.set_group_color(1, Some("#00FFFF".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    assert_eq!(buf[0], 255); // Dimmer
    assert_eq!(buf[1], 0); // Red
    assert_eq!(buf[2], 255); // Green
    assert_eq!(buf[3], 255); // Blue
    assert_eq!(buf[4], 0); // White
}

#[test]
fn test_live_engine_all_groups_color() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Spot 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    show.patch.fixtures.push(PatchedFixture {
        id: 2,
        groups: vec![2],
        name: "Spot 2".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Dimmer", "Red", "Green", "Blue"]}),
        universe: 1,
        address: 5,
    });

    engine.set_color_fade_speed(0.0);
    engine.set_group_effect(0, Some("full".to_string()), None, false);
    engine.set_all_groups_color(Some("magenta".to_string()), Some(&show));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    // Fixture 1 at addr 1 (channels 0..4)
    assert_eq!(buf[0], 255); // Dimmer
    assert_eq!(buf[1], 255); // Red
    assert_eq!(buf[2], 0); // Green
    assert_eq!(buf[3], 255); // Blue

    // Fixture 2 at addr 5 (channels 4..8)
    assert_eq!(buf[4], 255); // Dimmer
    assert_eq!(buf[5], 255); // Red
    assert_eq!(buf[6], 0); // Green
    assert_eq!(buf[7], 255); // Blue
}

#[test]
fn test_resolve_color_rgb_named_and_hex() {
    assert_eq!(resolve_color_rgb("#ff0000"), (255, 0, 0));
    assert_eq!(resolve_color_rgb("#00FF00"), (0, 255, 0));
    assert_eq!(resolve_color_rgb("#0000ff"), (0, 0, 255));
    assert_eq!(resolve_color_rgb("#€abc"), (255, 255, 255));
    assert_eq!(resolve_color_rgb("#12345z"), (255, 255, 255));
    assert_eq!(resolve_color_rgb("red"), (255, 0, 0));
    assert_eq!(resolve_color_rgb("green"), (0, 255, 0));
    assert_eq!(resolve_color_rgb("blue"), (0, 0, 255));
    assert_eq!(resolve_color_rgb("white"), (255, 255, 255));
    assert_eq!(resolve_color_rgb("unknown_color"), (255, 255, 255));
}

#[test]
fn test_color_fade_speed_simultaneous_fade() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    for i in 1..=3 {
        show.patch.fixtures.push(PatchedFixture {
            id: i,
            groups: vec![1],
            name: format!("Par {i}"),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
            universe: 1,
            address: ((i - 1) * 3 + 1) as u32,
        });
    }

    engine.set_color_fade_speed(0.2);
    engine.set_color_fade_order(None);
    engine.set_group_effect(1, Some("full".to_string()), None, false);

    // Initial state: white (255, 255, 255)
    // Start fade to blue (0, 0, 255)
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));

    // All fixtures must have active transitions with 0 delay and 0.2s duration
    assert_eq!(engine.fixture_color_transitions.len(), 3);
    for i in 1..=3 {
        let trans = engine.fixture_color_transitions.get(&i).unwrap();
        assert_eq!(trans.delay_secs, 0.0);
        assert_eq!(trans.duration_secs, 0.2);
        assert_eq!(trans.target_rgb, (0.0, 0.0, 255.0));
    }

    // Wait for transition to finish
    std::thread::sleep(std::time::Duration::from_millis(220));
    engine.compute_dmx(&show);

    let buf = engine.universe_buffers.get(&1).unwrap();
    // All 3 fixtures are now blue (0, 0, 255)
    for i in 0..3 {
        let offset = i * 3;
        assert_eq!(buf[offset], 0); // Red
        assert_eq!(buf[offset + 1], 0); // Green
        assert_eq!(buf[offset + 2], 255); // Blue
    }
}

#[test]
fn test_color_fade_left_order_staggered() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    for i in 1..=3 {
        show.patch.fixtures.push(PatchedFixture {
            id: i,
            groups: vec![1],
            name: format!("Par {i}"),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
            universe: 1,
            address: ((i - 1) * 3 + 1) as u32,
        });
    }

    engine.set_color_fade_speed(0.4);
    engine.set_color_fade_order(Some("left".to_string()));

    engine.set_group_color(1, Some("red".to_string()), Some(&show));

    let t1 = engine.fixture_color_transitions.get(&1).unwrap();
    let t2 = engine.fixture_color_transitions.get(&2).unwrap();
    let t3 = engine.fixture_color_transitions.get(&3).unwrap();

    // Fixture 1 has delay 0, Fixture 2 has mid delay, Fixture 3 has max delay
    assert_eq!(t1.delay_secs, 0.0);
    assert!(t2.delay_secs > t1.delay_secs);
    assert!(t3.delay_secs > t2.delay_secs);
}

#[test]
fn test_color_fade_wing_order_staggered() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    for i in 1..=3 {
        show.patch.fixtures.push(PatchedFixture {
            id: i,
            groups: vec![1],
            name: format!("Par {i}"),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
            universe: 1,
            address: ((i - 1) * 3 + 1) as u32,
        });
    }

    engine.set_color_fade_speed(0.4);
    engine.set_color_fade_order(Some("wing".to_string()));

    engine.set_group_color(1, Some("red".to_string()), Some(&show));

    let t1 = engine.fixture_color_transitions.get(&1).unwrap();
    let t2 = engine.fixture_color_transitions.get(&2).unwrap();
    let t3 = engine.fixture_color_transitions.get(&3).unwrap();

    // In a 3-fixture group, center is fixture 2 (local index 1), outer are 1 and 3
    // Center fixture 2 starts first (lowest delay)
    assert!(t2.delay_secs < t1.delay_secs);
    assert!(t2.delay_secs < t3.delay_secs);
}

#[test]
fn test_color_fade_center_order_staggered() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    for i in 1..=3 {
        show.patch.fixtures.push(PatchedFixture {
            id: i,
            groups: vec![1],
            name: format!("Par {i}"),
            fixture_type: serde_json::json!({}),
            mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
            universe: 1,
            address: ((i - 1) * 3 + 1) as u32,
        });
    }

    engine.set_color_fade_speed(0.4);
    engine.set_color_fade_order(Some("center".to_string()));

    engine.set_group_color(1, Some("red".to_string()), Some(&show));

    let t1 = engine.fixture_color_transitions.get(&1).unwrap();
    let t2 = engine.fixture_color_transitions.get(&2).unwrap();
    let t3 = engine.fixture_color_transitions.get(&3).unwrap();

    // In a 3-fixture group with center fade, outer fixtures 1 & 3 start first, center 2 starts last
    assert!(t1.delay_secs < t2.delay_secs);
    assert!(t3.delay_secs < t2.delay_secs);
    assert_eq!(t1.delay_secs, t3.delay_secs);
}

#[test]
fn test_hsv_to_rgb() {
    // Red (h = 0)
    let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
    assert_eq!(
        (r.round() as u8, g.round() as u8, b.round() as u8),
        (255, 0, 0)
    );

    // Green (h = 1/3)
    let (r, g, b) = hsv_to_rgb(1.0 / 3.0, 1.0, 1.0);
    assert_eq!(
        (r.round() as u8, g.round() as u8, b.round() as u8),
        (0, 255, 0)
    );

    // Blue (h = 2/3)
    let (r, g, b) = hsv_to_rgb(2.0 / 3.0, 1.0, 1.0);
    assert_eq!(
        (r.round() as u8, g.round() as u8, b.round() as u8),
        (0, 0, 255)
    );

    // Zero saturation produces pure white
    let (r, g, b) = hsv_to_rgb(0.5, 0.0, 1.0);
    assert_eq!(
        (r.round() as u8, g.round() as u8, b.round() as u8),
        (255, 255, 255)
    );
}

#[test]
fn test_concurrent_multiple_group_fades() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Group 1 Fixture".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    show.patch.fixtures.push(PatchedFixture {
        id: 2,
        groups: vec![2],
        name: "Group 2 Fixture".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
        universe: 1,
        address: 4,
    });

    engine.set_color_fade_speed(0.3);
    engine.set_color_fade_order(Some("left".to_string()));

    // Trigger fade on group 1 (red) and group 2 (cyan)
    engine.set_group_color(1, Some("red".to_string()), Some(&show));
    engine.set_group_color(2, Some("cyan".to_string()), Some(&show));

    let t1 = engine.fixture_color_transitions.get(&1).unwrap();
    let t2 = engine.fixture_color_transitions.get(&2).unwrap();

    assert_eq!(t1.target_rgb, (255.0, 0.0, 0.0));
    assert_eq!(t2.target_rgb, (0.0, 255.0, 255.0));
}

#[test]
fn test_mid_fade_color_redirection() {
    let mut engine = LiveEngine::new();
    let mut show = ShowData::default();

    show.patch.fixtures.push(PatchedFixture {
        id: 1,
        groups: vec![1],
        name: "Fixture 1".to_string(),
        fixture_type: serde_json::json!({}),
        mode: serde_json::json!({"channels": ["Red", "Green", "Blue"]}),
        universe: 1,
        address: 1,
    });

    engine.set_color_fade_speed(0.5);
    engine.set_color_fade_order(None);

    // Initial snap to red
    engine.set_color_fade_speed(0.0);
    engine.set_group_color(1, Some("red".to_string()), Some(&show));

    // Start 0.5s fade to blue
    engine.set_color_fade_speed(0.5);
    engine.set_group_color(1, Some("blue".to_string()), Some(&show));

    // After 100ms, redirect to green
    std::thread::sleep(std::time::Duration::from_millis(100));
    engine.set_group_color(1, Some("green".to_string()), Some(&show));

    let t = engine.fixture_color_transitions.get(&1).unwrap();
    assert_eq!(t.target_rgb, (0.0, 255.0, 0.0));
    // Start RGB should be intermediate (interpolated red-blue mix), not snap to white or blue
    assert!(t.start_rgb.0 > 0.0); // partial red
    assert!(t.start_rgb.2 > 0.0); // partial blue
}
